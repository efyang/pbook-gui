# pbook-gui

This project aims to provide a gui to allow users to select and download files from the [free-programming-books](https://github.com/vhf/free-programming-books) repository (currently only supporting the english version) in parallel.

### Features (very much WIP)
- parser for github page ☑
    - page -> list of categories of downloads ☑
- gtk gui ☐
    - adequate directory structure ☑
    - fairly easily themable ☑
    - swappable themes while in gui ☐
    - categories on side ☐
    - list of downloads ☐
        - name ☐
        - progress bar ☐
        - ETA ☐
        - speed (use XiB notation) ☐
- parallel downloads ☐
    - thread pool based ☐
    - number of threads changeable while executing ☐
    - pausable ☐
    - low cpu usage ☐
    - speed should be at max possible ☐
    - Optional - if available, use coroutines
- logging ☐
    - just print ☑
    - write all errors to logfile (Optional)

### Architecture

##### General
```
+---------------+
|               |                      spawn         +------------+
|  Main Thread  |               +------------------->+  Download  |
|               |               |                    | ThreadPool |
+-------+-------+               |                    +-----+-----^+
        |                       |                          |     |
        |                       |                          |     |
        |             +---------+--------+            +----v---+ |
        |    spawn    |                  |            |Progress| |
        +------------>+  GUI/Downloader  <------------+ Update | |
        |             |   Comm Handler   |            | Channel| |
 (main  |             |                  +----+       +--------+ |
becomes |             +----------+--^----+    |                  |
  GUI)  |                        |  |         |    +-------------+-+
        |                        |  |         |    | Channel with  |
     +--v--+    +--------------+ |  |         |    |  individual   |
     |     <----+Update Channel+-+  |         +---->thread commands|
     | GUI |   +------------------+ |              | (e.g. pause,  |
     |     +--->UI Command Channel+-+              |  change dir)  |
     +-----+   +------------------+                +---------------+
```

* All channels should use normal std builtin mpsc channels unless they become a bottleneck, otherwise use comm::spsc

##### GUI
* Tree view for representation of the categories
* RadioBox of downloads
    * Right click on each item -> Context Menu with pause/resume/disable

##### Download ThreadPool
```
     +-----------+----------+
     |           |          |
+----v-------+   |       +--+---+ +----------------+
|Status Recv +---------->+Thread+-+Job Recv Channel<-+
+----+-------+   +-----+ +------+ +----------------+ |
     |      Initial  | +-----|                       |
+----v----+  Spawn   | | +------+ +----------------+ |
|Scheduler+------------->+Thread+-+Job Recv Channel<-+
+----+----+          | | +------+ +----------------+ |
     |               | +-----|                       |
     |               |   +------+ +----------------+ |
     |               +-->+Thread+-+Job Recv Channel<-+
     |                   +------+ +----------------+ |
     |                                               |
     +-----------------------------------------------+
         Jobs/Commands sent to individual threads
```
* Job threads are initially spawned by the scheduler
* Scheduler has list of Downloads, and if there are idle threads it sends it to that  thread
* Scheduler maintains HashMap of all threadids (Maybe use a Vec with preset capacity)
    * References: Handle to job channel
* Scheduler maintains HashMap of idle threadids
    * References: Handle to job channel
* All threads have access to status recv channel, and send threadid when done with their job

<!--  ☐
 ☑-->
<!--
* enabled enum ->
	* Enabled(progressamnt) -> on download check if progressamnt is 100 -> do not redownload
	* Disabled
* Have a download struct
	* title
	* url
	* enabled enum
* Main gui thread:
	* HashMap<threadid, download struct> -> shared between all threads via arc<mutex>
	* Download progress updater updates the main hashmap
	* Gui thread just reads the hashmap and renders accordingly
* Ideally have main gui be easily modifiable for uses other than pdfs
	* Split up window sections/tabs
		1. pdf chooser/browser
		2. ongoing downloads -> should easy be able to be repurposed for similar tasks
* Threadpool for downloads -> main gui thread distributes work
	* max 4 parallel downloads/maximum os threads -> whichever is smaller
	* threadpool is Hashmap<thread, bool> (bool is whether working or not)
	* Check for open threads on each gui update loop
-->
