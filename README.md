# pbook-gui

This project aims to provide a gui to allow users to select and download files from the [free-programming-books](https://github.com/vhf/free-programming-books) repository (currently only supporting the english version) in parallel. Ideally, the parser can be generalized to accept or try to accept other similar pages and give users the ability to download lists of files that they choose while making the most of their connection.

### Features (very much WIP)
- parser for github page ☑
    - page -> list of categories of downloads ☑
- gtk gui ☑
    - adequate directory structure ☑
    - fairly easily themable ☑
    - swappable themes while in gui ☐
    - button to enable all ☑
    - button to disable all ☑
    - categories on side ☑
    - list of downloads ☑
        - name ☑
            - ellipsized (needs PR) ☑
        - progress bar ☑
        - ETA ☑
        - speed (use XiB notation) ☑
        - updating ☑
        - no lag (mostly) ☑
        - right click context menu ☐
            - pause ☐
            - open file ☐
            - stop file ☐
- parallel downloads ☑
    - thread pool based ☑
    - number of threads changeable while executing ☐
    - pausable ☐
    - low cpu usage ☑
    - speed should be at max possible ☑
    - Optional - if available, use coroutines
- logging ☐
    - just print ☑
    - write all errors to logfile (Optional)

### Architecture

##### General
```
+---------------+
|               |                                    +------------+
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

* ~~Thread number unchangeable atm~~ Functionality added with [this PR](https://github.com/frewsxcv/rust-threadpool/pull/17)

##### GUI
* Enable 
* Tree view for representation of the categories ☑
* RadioBox of downloads ☑
    * Right click on each item -> Context Menu with pause/resume/disable ☐
