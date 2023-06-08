#!/usr/bin/python3

import os, sys, glob
def get_last_log():
    if sys.platform == "linux":
        path = os.path.join(os.path.expanduser('~'), ".local", "share", "yarp", "logs", "*.log")
    elif sys.platform == "win32":
        path = os.path.join(os.path.expanduser('~'), "AppData", "Roaming", "yarp", "logs", "*.log")

    files = glob.glob(path)
    times = {}

    for x in files:
        time = os.stat(x).st_mtime
        times[x] = time

    with open(max(times), "r") as file:
        content = file.read()
        return content

if __name__ == "__main__":
    print(get_last_log())
