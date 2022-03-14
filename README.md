# SPUR
Simple recorder with webcam overlay

## Installation
- Downlaod the spur binary from the latest release.
- Since this project heavily uses gstreamer, whose libraries are dynamically linked, you will have to download the required gstreamer libraries. The following command should work on debian based machines, but you can [refer this article](https://gstreamer.freedesktop.org/documentation/installing/on-linux.html?gi-language=c) for other linux distros
  _I have only tested spur on ubuntu so far. But I would be happy to help if you are keen to test it's support on your distro_
  
```shell
sudo apt-get update
```
```shell
sudo apt-get install libgstreamer1.0-dev libgstreamer-plugins-base1.0-dev libgstreamer-plugins-bad1.0-dev gstreamer1.0-plugins-base gstreamer1.0-plugins-good gstreamer1.0-plugins-bad gstreamer1.0-plugins-ugly gstreamer1.0-libav gstreamer1.0-doc gstreamer1.0-tools gstreamer1.0-x gstreamer1.0-alsa gstreamer1.0-gl gstreamer1.0-gtk3 gstreamer1.0-qt5 gstreamer1.0-pulseaudio
```


- Mark it as executable by running `chmod +x spur` on the binary
- Run the setup `./spur setup`. This sets the directory where the videos will be stored
- Use `./spur -h` or `./spur <command> -h` to see the available options 
> **Note:**  Currently only supports local recordings. The stream functionality for remote recording is still a WIP

### Example

After running `./spur setup` if there were no errors you can run\
```
./spur record --filename=testRecording
``` 
Should start the recording while showing you a sticky overlay of your webcam preview.
If you use an external webcam like me, make sure that it is plugged in when you do so, as the program will crash if a camera is not found. 

**Once you are done recording, you can stop the recording session by typing `end` into the terminal** 

This is better than using `Ctrl + C` and killing the terminal process as that would result in some parts of the recording not being correctly saved.

---
_If you face any problems while trying to run this project, consider raising an issue or reaching out to me directly._ PRs are welcome too😄