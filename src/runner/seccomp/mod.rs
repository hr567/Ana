use std::ffi;

use nix;
mod libseccomp;
use libseccomp::*;

type Syscall = u32;

pub fn syscall(name: &str) -> i32 {
    let name = ffi::CString::new(name).unwrap();
    let syscall = unsafe { seccomp_syscall_resolve_name(name.as_ptr()) };
    if syscall < 0 {
        panic!("No such syscall")
    }
    syscall
}

#[repr(u32)]
pub enum ScmpAct {
    Kill = SCMP_ACT_KILL,
    Allow = SCMP_ACT_ALLOW,
}

#[repr(u8)]
#[allow(dead_code)]
pub enum ScmpCmp {
    NE = 1,
    LT = 2,
    LE = 3,
    EQ = 4,
    GE = 5,
    GT = 6,
}

pub struct ScmpArg {
    _inner: scmp_arg_cmp,
}

impl ScmpArg {
    pub fn new(arg_n: usize, op: ScmpCmp, value: u64) -> ScmpArg {
        ScmpArg {
            _inner: scmp_arg_cmp {
                arg: arg_n as u32,
                op: op as u32,
                datum_a: value,
                datum_b: 0,
            },
        }
    }
}

pub struct ScmpCtx {
    ctx: scmp_filter_ctx,
}

impl ScmpCtx {
    pub fn new() -> ScmpCtx {
        ScmpCtx {
            ctx: unsafe { seccomp_init(ScmpAct::Kill as u32) },
        }
    }

    pub fn add_rule(
        &self,
        act: ScmpAct,
        syscall: Syscall,
        args: Vec<ScmpArg>,
    ) -> Result<(), nix::errno::Errno> {
        if unsafe {
            seccomp_rule_add_array(
                self.ctx,
                act as u32,
                syscall as i32,
                args.len() as u32,
                args.as_ptr() as *const scmp_arg_cmp,
            ) == 0
        } {
            Ok(())
        } else {
            Err(nix::errno::from_i32(nix::errno::errno()))
        }
    }

    pub fn whitelist(&self, syscall: Syscall, args: Vec<ScmpArg>) -> Result<(), nix::errno::Errno> {
        self.add_rule(ScmpAct::Allow, syscall, args)
    }

    // pub fn blacklist(&self, syscall: Syscall, args: Vec<ScmpArg>) -> Result<(), nix::errno::Errno> {
    //     self.add_rule(ScmpAct::Kill, syscall, args)
    // }

    pub fn load(&self) -> Result<(), nix::errno::Errno> {
        if unsafe { seccomp_load(self.ctx) } == 0 {
            Ok(())
        } else {
            Err(nix::errno::from_i32(nix::errno::errno()))
        }
    }
}

impl Drop for ScmpCtx {
    fn drop(&mut self) {
        unsafe {
            seccomp_release(self.ctx);
        }
    }
}
