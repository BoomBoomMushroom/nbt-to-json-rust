# NBT to JSON (using Rust)

This is my solution to slow nbt loading in Python. When I was working on my Minecraft World Wallpaper the NBT library I was using (NBTLib) was one of the biggest time users when building the mesh (basically the entire program!) So I built this tool in rust to turn nbt data into nice and pretty json!  

Use:  

- nbt-to-json-rust.exe --input=INPUT_FILE --output=OUTPUT_FILE  
The input file is required and this program will try its best to load it and decode it.
The output file is OPTIONAL, the program will always print the json into the console so you can use something like python subprocesses to get the console output