# fuzzy-match
Teaching assistant tool for recording marks, featuring fuzzy name matching.

## Getting started

1. Install [cargo-web] 
 
        $ cargo install cargo-web
 
2. To run the tool locally using browser at `http://localhost:8000` 
 
        $ cargo web start

 
3. To build deployable [WebAssembly] binaries packed together with the .html .js files 

        $ cargo web deploy

4. Other [cargo-web] commands

    Run tests
    
        $ cargo web test

    Build [WebAssembly] binaries 
 
        $ cargo web build

[cargo-web]: https://github.com/koute/cargo-web
[WebAssembly]: https://en.wikipedia.org/wiki/WebAssembly
