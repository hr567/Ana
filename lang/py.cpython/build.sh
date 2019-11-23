echo "#!/usr/bin/env python3" > $EXECUTABLE_FILE
cat $SOURCE_FILE >> $EXECUTABLE_FILE
chmod +x $EXECUTABLE_FILE