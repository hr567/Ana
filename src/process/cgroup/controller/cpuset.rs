use std::ops::Bound::*;
use std::io;
use std::future::Future;
use std::task::{Poll, Waker};
use std::sync::{Arc, Mutex};
use std::pin::Pin;
use std::collections::BTreeSet;
use std::path::Path;
use std::marker::PhantomData;
use super::Controller;
use std::fs::{create_dir, write, read_to_string};
use super::*;
use lazy_static::lazy_static;

lazy_static! {
    static ref CPUSET_ALLOCATOR: Arc<Mutex<CpusetAllocator>> = Arc::new(Mutex::new(CpusetAllocator::new()));
    static ref SHARED_STATUS: Arc<Mutex<SharedStatus>> = Arc::new(Mutex::new(SharedStatus {
        waker: Vec::new(),
    }));
}

#[derive(Debug, Clone)]
pub enum Error {
    CpuNotEnough,
    InvalidCpuset,
}

pub struct CpusetAllocatorFuture {
    allocator: Arc<Mutex<CpusetAllocator>>,
    num_of_cpu: u32,
    shared_states: Arc<Mutex<SharedStatus>>,
}

#[derive(Debug, Clone)]
pub struct SharedStatus {
    waker: Vec<Waker>,
}


impl Future for CpusetAllocatorFuture {
    type Output = Vec<(u32, u32)>;
    fn poll(self: Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut shared_status = self.shared_states.lock().unwrap();
        let mut allocator = self.allocator.lock().unwrap();
        match allocator.allocate(self.num_of_cpu) {
            Ok(data) => {
                Poll::Ready(data)
            },
            Err(Error::CpuNotEnough) => {
                shared_status.waker.push(cx.waker().clone());
                Poll::Pending
            },
            Err(_) => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct CpusetAllocator {
    avail: BTreeSet<(u32, u32)>,
    avail_cpu_num: u32,
    cpu_num: u32,
}

impl CpusetAllocator {
    fn new() -> CpusetAllocator {
        let cpus = num_cpus::get() as u32;
        let mut avail = BTreeSet::new();
        avail.insert((0 as u32, (cpus - 1) as u32));
        Self {
            avail,
            avail_cpu_num: cpus,
            cpu_num: cpus,
        }
    }

    fn allocate(&mut self, num_of_cpu: u32) -> Result<Vec<(u32, u32)>, Error> {
        if self.avail_cpu_num < num_of_cpu {
            return Err(Error::CpuNotEnough);
        }
        let mut to_insert: Vec<(u32, u32)> = Vec::new();
        let mut cpu_remain = num_of_cpu;
        let mut cpus_to_allocate = Vec::new();
        self.avail = self.avail.iter().filter(|(start, end)| {
            let len = end - start + 1;
            let allocated = std::cmp::min(len, cpu_remain);
            cpu_remain -= allocated;
            if allocated < len {
                to_insert.push((start + allocated, *end));
            }
            if allocated > 0 {
                cpus_to_allocate.push((*start, start + allocated - 1));
            }
            allocated == 0
        }).cloned().collect();

        to_insert.into_iter().for_each(|(start, end)| {
            self.avail.insert((start, end));
        });
        self.avail_cpu_num -= num_of_cpu;
        Ok(cpus_to_allocate)
    }

    fn union(&mut self, mut other: (u32, u32)) {
        let prev = self.avail.range((Unbounded, Included(&other))).next_back();
        let next = self.avail.range((Excluded(&other), Unbounded)).next();
        let mut need_remove = Vec::new();

        if let Some(rng) = prev {
            if rng.1 + 1 >= other.0 {
                self.avail_cpu_num -= rng.1 - rng.0 + 1;
                other.0 = rng.0;
                need_remove.push(rng.clone());
            }
        }

        if let Some(rng) = next {
            if rng.0 <= other.1 + 1 {
                self.avail_cpu_num -= rng.1 - rng.0 + 1;
                other.1 = rng.1;
                need_remove.push(rng.clone());
            }
        }
        need_remove.iter().for_each(|it| {
            self.avail.remove(it);
        });
        self.avail.insert((other.0, other.1));
        self.avail_cpu_num += other.1 - other.0 + 1;
    }

    fn release<'a, I>(&mut self, container: I) -> Result<(), Error>
    where
        I: 'a + IntoIterator<Item = &'a (u32, u32)>
    {
        container.into_iter().try_for_each(|(start, end)| {
            if *start > *end || *end > self.cpu_num {
                return Err(Error::InvalidCpuset)
            }
            self.union((*start, *end));
            Ok(())
        })
    }
}

#[derive(Debug)]
pub struct CpusetData {
    allocator: Arc<Mutex<CpusetAllocator>>,
    shared_status: Arc<Mutex<SharedStatus>>,
    data: Vec<(u32, u32)>,
}

impl Drop for CpusetData {
    fn drop(&mut self) {
        log::debug!("try to release cpuset {:?}", self.data);
        let _ = self.allocator.lock().unwrap().release(&self.data)
            .map_err(|e| {
                log::error!("release cpuset {:?} failed, err: {:?}", &self.data, e);
                e
            });
        let mut shared_status = self.shared_status.lock().unwrap();
        shared_status.waker.drain(..).for_each(|waker| {
            waker.wake()
        });
        shared_status.waker = Vec::new();
    }
}

pub struct CpusetController<'a, T: 'a + AsRef<Path>> {
    inner: T,
    _mark: PhantomData<&'a ()>,
}

impl<'a, T: 'a + AsRef<Path>> CpusetController<'a, T> {
    pub async fn allocate(&mut self, num_of_cpu: u32) -> io::Result<Arc<CpusetData>> {
        let future = CpusetAllocatorFuture {
            allocator: CPUSET_ALLOCATOR.clone(),
            shared_states: SHARED_STATUS.clone(),
            num_of_cpu,
        };
        let data = future.await;
        self.write_cpuset(&data)?;
        Ok(Arc::new(CpusetData {
            allocator: CPUSET_ALLOCATOR.clone(),
            shared_status: SHARED_STATUS.clone(),
            data
        }))
    }

    pub fn allocated(&self) -> io::Result<Vec<(u32, u32)>> {
        let file = CpusetFile {
            inner: self.inner.as_ref().join("cpuset.cpus"),
            _marker: PhantomData
        };

        file.read()
    }

    fn write_cpuset(&mut self, data: &Vec<(u32, u32)>) -> io::Result<()> {
        let mut file = CpusetFile {
            inner: self.inner.as_ref().join("cpuset.cpus"),
            _marker: PhantomData
        };
        let mut mem_file = CpusetFile {
            inner: self.inner.as_ref().join("cpuset.mems"),
            _marker: PhantomData
        };
        mem_file.write(&vec![(0, 0)]);
        file.write(data)?;
        Ok(())
    }
}

impl<'a> Controller<'a> for CpusetController<'a, PathBuf> {
    const NAME: &'static str = "cpuset";

    fn from_ctx(context: &Context) -> CpusetController<PathBuf> {
        CpusetController {
            inner: Context::root().join(Self::NAME).join(&context.name),
            _mark: PhantomData,
        }
    }

    fn initialize(&self) -> io::Result<()> {
        match create_dir(&self.inner) {
            Ok(_) => {}
            Err(ref e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }

        Ok(())
    }
}


impl<'a, T: 'a + AsRef<Path>> AsRef<Path> for CpusetController<'a, T> {
    fn as_ref(&self) -> &Path {
        self.inner.as_ref()
    }
}

pub struct CpusetFile<'a, T: 'a + AsRef<Path>> {
    inner: T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: 'a + AsRef<Path>> AttrFile<'a, Vec<(u32, u32)>, Vec<(u32, u32)>> for CpusetFile<'a, T> {
    fn read(&self) -> io::Result<Vec<(u32, u32)>> {
        let attr: Vec<(u32, u32)> = read_to_string(&self.inner)?
            .trim()
            .split(",")
            .map(|s| {
                if s.contains("-") {
                    let v: Vec<u32> = s.split("-").map(|s| s.parse::<u32>().unwrap()).collect();
                    (v[0], v[1])
                } else {
                    let cpu: u32 = s.parse().unwrap();
                    (cpu, cpu)
                }
            })
            .collect();
        Ok(attr)
    }

    fn write(&mut self, attr: &Vec<(u32, u32)>) -> io::Result<()> {
        let mut ans = Vec::new();
        for (start, end) in attr {
            if start == end {
                ans.push(format!("{}", start))
            } else {
                ans.push(format!("{}-{}", start, end))
            }
        }
        write(&self.inner, ans.join(","))?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Mutex};
    use super::*;

    async fn try_allocate(cpuset_allocator: Arc<Mutex<CpusetAllocator>>, num_of_cpu: u32) -> Vec<(u32, u32)> {
        println!("try to allocate {} cpu", num_of_cpu);
        let future = CpusetAllocatorFuture {
            allocator: cpuset_allocator.clone(),
            shared_states: Arc::new(Mutex::new(SharedStatus {waker: Vec::new()})),
            num_of_cpu,
        };
        future.await
    }

    fn show_avail(cpuset_allocator: Arc<Mutex<CpusetAllocator>>) {
        println!("cpuset_allocator avail is {:?}", cpuset_allocator.lock().unwrap().avail);
    }

    fn release(cpuset_allocator: Arc<Mutex<CpusetAllocator>>, data: &Vec<(u32, u32)>) -> Result<(), Error> {
        cpuset_allocator.lock().unwrap().release(data)
    }

    #[tokio::test]
    async fn test_allocator() -> io::Result<()> {
        let cpuset_allocator = Arc::new(Mutex::new(CpusetAllocator::new()));

        let first_allocated = try_allocate(cpuset_allocator.clone(), 1).await;
        println!("allocated data is {:?}", first_allocated);
        show_avail(cpuset_allocator.clone());

        let second_allocated = try_allocate(cpuset_allocator.clone(), 3).await;
        println!("allocated data is {:?}", second_allocated);
        show_avail(cpuset_allocator.clone());

        let third_allocated = try_allocate(cpuset_allocator.clone(), 1).await;
        println!("allocated data is {:?}", third_allocated);
        show_avail(cpuset_allocator.clone());

        let fourth_allocated = try_allocate(cpuset_allocator.clone(), 1).await;
        println!("allocated data is {:?}", fourth_allocated);
        show_avail(cpuset_allocator.clone());


        release(cpuset_allocator.clone(), &first_allocated);
        show_avail(cpuset_allocator.clone());

        release(cpuset_allocator.clone(), &third_allocated);
        show_avail(cpuset_allocator.clone());

        release(cpuset_allocator.clone(), &second_allocated);
        show_avail(cpuset_allocator.clone());

        release(cpuset_allocator.clone(), &fourth_allocated);
        show_avail(cpuset_allocator.clone());

        Ok(())
    }
}


