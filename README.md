### pbook-gui

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

* Have gtktheme file that specifies a gtk theme, on start read and then combine with gtk.css and then parse -- use main git branch for feature
