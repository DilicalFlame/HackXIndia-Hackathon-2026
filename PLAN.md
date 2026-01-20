1. make a generic template with svelte + tauri to ensure cross platform compatibility
2. build the backend with two cases
    - if the detected enviroment is tauri, use tauri's rust API's to do file watches etc
    - if the detected enviroment is not tauri, simply ask user to upload files and make a virtual file system on the web local storage
3. don't focus on design, build the design quickly using shadcn components
4. compile for windows desktop and release the exe
5. deploy web on vercel
6. submit on unstop