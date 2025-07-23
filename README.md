
# Native Screen Renderer for MaSzyna

**Native Screen Renderer (NSR)** is a modern solution for implementing digital displays in the [MaSzyna](https://eu07.pl/) train simulator.  
The project relies on dynamically loaded libraries (`.dll`, `.so`, `.dylib`) and can be implemented in any language that compiles to native machine code (Rust, C, C++, Pascal, etc.).

This repository contains a **reference implementation written in Rust (2024 edition)**.

---

## 🔧 NSR Architecture

NSR is based on runtime function loading using `DllImport`/`DllExport` between the simulator and an external rendering library.

The library **must export 3 functions** using the **C ABI** and **without any name mangling** — exactly as in standard C.

---

## 📌 Required Functions

The following functions **MUST** be exported exactly in the following form:

```c
void Init(const char* path);
```
Called once when the cab is initialized — e.g., after vehicle load, cab switch, or simulation start.  
The `path` parameter contains the full path to the vehicle directory, e.g.:  
`D:\MaSzyna\dynamic\pkp\e186_v2\`

---

```c
unsigned char* Render(const KeyValue* pairs, size_t count, size_t* out_len);
```
Called once per frame. Receives a list of `<key, value>` pairs representing the current simulation state (see Python API).

The function should:
- Parse required values from the dictionary: `texW`, `texH`, `texFormat`
- Generate a `char*` buffer representing the output image in **RGB** or **RGBA** format
- Set `*out_len` to the buffer size (`width * height * channels`)
- Return a pointer to the data (allocated with `malloc()` or equivalent)

The image is expected to be in **row-major Z-order** layout — i.e., pixels arranged row by row:
```
row 1: pixel[0], pixel[1], pixel[2]...
row 2: pixel[W], pixel[W+1], ...
```

Each pixel consists of 3 or 4 bytes (RGB or RGBA), for example:
```
[ R G B ] [ R G B ] ...    (for RGB)  
[ R G B A ] [ R G B A ] ... (for RGBA)
```

---

```c
CommandData* GetCommands(size_t* count);
```
Called immediately after `Render()`.  
Returns a list of commands to be executed during this frame (e.g., `batterytoggle null null`).  
By default, this function may return an empty array.

---

## 📦 Support Structures (C ABI)

All data structures exchanged between DLL and host must follow **C memory layout**:

```c
typedef struct {
    const char* key;
    const char* value;
} KeyValue;
```
Used in `Render()` to pass simulation variables.

```c
typedef struct {
    const char* command;
    double param1;
    double param2;
} CommandData;
```
Used in `GetCommands()` to return a list of pending vehicle commands.

---

## 🧹 Memory Management

Memory returned by `Render()` or `GetCommands()` **must not be freed by the simulator**.  
If you allocate memory using `malloc()` or `CString::into_raw()`, your library should provide cleanup functions such as:

```c
void FreeBuffer(void* ptr);
void FreeCommands(CommandData* ptr, size_t count);
```

---

## 📁 Additional Info

The `src/` directory contains a minimal example implementation of the above functions.

---

### 🔗 Reference Documentation

- Simulation variable keys (`KeyValue`):  
  📖 https://wiki.eu07.pl/index.php/Python

- List of vehicle commands (`CommandData`):  
  📖 https://wiki.eu07.pl/index.php/Komendy_pojazdu

---

## ✅ Requirements

- Rust 1.88+ (2024 edition)
- `crate-type = "cdylib"` in `Cargo.toml`
- All functions must use `extern "C"` and `#[unsafe(no_mangle)]`

---

## 🧱 Output Format

After building, your library will output a platform-specific binary:

| Platform | File extension         |
|----------|------------------------|
| Windows  | `.dll`                 |
| Linux    | `.so`                  |
| macOS    | `.dylib`               |

The DLL should be loaded by MaSzyna dynamically at cab startup.  
It is recommended to include platform-specific binaries in the vehicle directory so users can run the same vehicle on different systems.

The filename should follow this convention:
- Platform suffix: `win`, `linux`, `macos`
- Architecture suffix: `_x86_64`, `_arch64`

The `x86_32` architecture is **not supported** by the simulator.

---

### 📄 Example Filenames:

- `mainscreenrenderer_win_x86_64.dll`
- `mainscreenrenderer_linux_x86_64.so`
- `mainscreenrenderer_linux_arch64.so`
- `mainscreenrenderer_macos_x86_64.dylib`
