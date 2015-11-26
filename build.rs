fn main() {
    //get_dep_dir();
    if cfg!(windows) {
        let depdir; 
        if cfg!(target_pointer_width = "64") {
            //64 bit
            depdir = "./windows-deps/x86_64";
        } else {
            //32 bit
            depdir = "./windows-deps/x86";
        }  
        println!("cargo:rustc-link-search={}", depdir);
    } else { 
        //do nothing
    }
}
