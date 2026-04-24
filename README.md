# autoclicker.exe  
Scripted Autoclicker Tool on Rust  
For legacy software automation, game cheating, etc  

<img width="578" height="395" alt="image" src="https://github.com/user-attachments/assets/ec0be864-ac0e-482b-8c84-9b98641be523" />  

### Building the program
You will need to set up the environment if you'd like to compile by your own:

```cmd
rustup install 1.75                                  
rustup override set 1.75
```

For x86 32-bit architecture:
```cmd
rustup target add i686-pc-windows-msvc
cargo build --release --target i686-pc-windows-msvc
```

For x86 64-bit architecture:
```cmd
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```
