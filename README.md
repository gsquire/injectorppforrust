# Injectorpp for rust

[![CI](https://github.com/microsoft/injectorppforrust/actions/workflows/ci.yml/badge.svg)](https://github.com/microsoft/injectorppforrust/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/injectorpp.svg)](https://crates.io/crates/injectorpp)
[![Documentation](https://docs.rs/injectorpp/badge.svg)](https://docs.rs/injectorpp)

Injectorpp for rust is a crate that allows you to change rust function behavior at runtime without adding additional traits or code changes. It introduces an innovative and easy way to write rust unit tests.

# Why injectorpp

When working with a large codebase, a common challenge in writing unit tests is managing dependencies. Elements such as disk I/O, network operations, and even singleton or static functions can cause the code non-unit testable.

Traditionally, writing effective unit tests for such code requires refactoring the production code first. This process may involve introducing additional traits, even when there is only one implementation in production. Consequently, numerous traits are created solely for testing purposes, rather than for actual production use.

For example, to write tests for below code:

```rust
fn try_repair() -> Result<(), String> {
    if let Err(e) = fs::create_dir_all("/tmp/target_files") {
        // Failure business logic here

        return Err(format!("Could not create directory: {}", e));
    }

    // Success business logic here

    Ok(())
}
```

The code itself is clean and readable but it's not unit testable as `fs::create_dir_all` introduces a dependency on disk. Traditionally, it is necessary to refactor the code to enable the passing of a trait into the function, thereby abstracting away the `fs::create_dir_all()` operation. Or you have to setup the environment to make sure `/tmp/target_files` exists to cover the success path.

With injectorpp, you can write tests without needing to modify the production code solely to make it unit testable or setup environment:

```rust
let mut injector = InjectorPP::new();
injector
    .when_called(injectorpp::func!(fn (fs::create_dir_all)(&'static str) -> std::io::Result<()>)
    .will_execute(injectorpp::fake!(
        func_type: fn(path: &str) -> std::io::Result<()>,
        when: path == "/tmp/target_files",
        returns: Ok(()),
        times: 1
    ));

assert!(try_repair().is_ok());
```

Notice that `try_repair()` is not changed whereas `fs::create_dir_all` is successfully abstracted away. No external dependency, all happen in-memory.

The above config make sure when `fs::create_dir_all` is called with `/tmp/target_files` as its `path` parameter, it will always return `Ok(())` and it's expected to be executed only once.

This approach eliminates the need to make a trait solely for testing purposes. It also ensures that previously non-unit testable code is now unit testable.

# Supported Platform
- OS: Linux and Windows
- Arch: arm64 and amd64

# Usage

Add `injectorpp` to the `Cargo.toml`:

```toml
[dev-dependencies]
injectorpp = "0.4.0"
```

Below `profile.test` config is recommended to make sure `injectorpp` working correctly in tests. If you have workspace, make sure add this on the top level of `Cargo.toml`:

```toml
[profile.test]
opt-level = 0
debug = true
lto = false
codegen-units = 1
incremental = false
```

Import injectorpp in the code:

```rust
use injectorpp::interface::injector::*;
```

Below are multiple ways to config the function behavior.

## `will_return_boolean`

If the function only returns boolean and you only want to make it constantly returns a specific boolean value, you can use `will_return_boolean`:

```rust
let mut injector = InjectorPP::new();
injector
    .when_called(injectorpp::func!(fn (Path::exists)(&Path) -> bool))
    .will_return_boolean(true);
```

Above code will make `Path::exists` always return true.

## `will_execute`

For complex scenarios, `will_execute` is the major feature to use.

In `will_execute` there are different options:

```rust
func_type: // Required. The signature of the function to fake.
when: // Optional. A condition check for the parameters of the function to fake.
assign: // Optional. Use to set values to reference variables of the function to fake.
returns: // Required for the function has return. Specify what the return value should be.
times: // Optional. How many times the function should be called. If the value is not satisfied at the end of the test, the test will fail.
```

A simple example:

```rust
#[test]
fn test_will_execute_when_fake_file_dependency_should_success() {
    let mut injector = InjectorPP::new();
    injector
        .when_called(injectorpp::func!(fn (Path::exists)(&Path) -> bool))
        .will_execute(injectorpp::fake!(
            func_type: fn(_path: &Path) -> bool,
            returns: true
        ));

    let test_path = "/path/that/does/not/exist";
    let result = Path::new(test_path).exists();

    assert_eq!(result, true);
}
```

Below is a more complex example. The function has generic type. The fake only takes effect when a given generic type is hit.

```rust
#[test]
fn test_will_execute_when_fake_generic_function_multiple_types_should_success() {
    let mut injector = InjectorPP::new();
    injector
        .when_called(injectorpp::func!(
            fn (complex_generic_multiple_types_func)(&'static str, bool, i32) -> String
        ))
        .will_execute(injectorpp::fake!(
            func_type: fn(a: &str, b: bool, c: i32) -> String,
            when: a == "abc" && b == true && c == 123,
            returns: "Fake value".to_string(),
            times: 1
        ));

    let actual_result = complex_generic_multiple_types_func("abc", true, 123);

    // This call should not be counted as the types are different from the fake_closure.
    complex_generic_multiple_types_func(1, 2, 3);

    assert_eq!(actual_result, "Fake value".to_string());
}
```

Below is an example for assigning the values to the reference parameters:

```rust
#[test]
fn test_will_execute_when_fake_multiple_reference_param_function_should_success() {
    let mut injector = InjectorPP::new();
    injector
        .when_called(injectorpp::func!(
            fn (multiple_reference_params_func)(&mut i32, &mut bool) -> bool
        ))
        .will_execute(injectorpp::fake!(
            func_type: fn(a: &mut i32, b: &mut bool) -> bool,
            assign: { *a = 6; *b = true },
            returns: true,
            times: 1
        ));

    let mut value1 = 0;
    let mut value2 = false;

    let result = multiple_reference_params_func(&mut value1, &mut value2);

    assert_eq!(value1, 6);
    assert_eq!(value2, true);
    assert_eq!(result, true);
}
```

Below is an example for faking a method:

```rust
#[test]
fn test_will_execute_when_fake_method_with_parameter_should_success() {
    let mut injector = InjectorPP::new();
    injector
        .when_called(injectorpp::func!(fn (Foo::add)(&Foo, i32) -> i32))
        .will_execute(injectorpp::fake!(
            func_type: fn(f: &Foo, value: i32) -> i32,
            when: f.value > 0,
            returns: f.value * 2 + value * 2
        ));

    let foo = Foo::new(6);
    let result = foo.add(3);

    assert_eq!(result, 18);
}
```

The fake can be limited to a given scope:

```rust
#[test]
fn test_will_execute_when_fake_generic_function_single_type_can_recover() {
    {
        let mut injector = InjectorPP::new();
        injector
            .when_called(injectorpp::func!(
                fn (complex_generic_single_type_always_fail_func)(&'static str) -> std::io::Result<()>
            ))
            .will_execute(injectorpp::fake!(
                func_type: fn(path: &str) -> std::io::Result<()>,
                when: path == "/not/exist/path",
                returns: Ok(()),
                times: 1
            ));

        let actual_result = complex_generic_single_type_always_fail_func("/not/exist/path");

        assert!(actual_result.is_ok());
    }

    let actual_result = complex_generic_single_type_always_fail_func("/not/exist/path");

    assert!(actual_result.is_err());
}
```

More examples can be found [here](tests/will_execute.rs).

## `will_execute_raw`

`will_execute_raw` allows to fully customize the function behavior. A custom function or closure can be used to replace the original function.

Below is an example for using custom function:

```rust
pub fn fake_path_exists() -> bool {
    println!("fake_path_exists executed.");
    true
}

#[test]
fn test_will_execute_raw_when_fake_file_dependency_should_success() {
    let mut injector = InjectorPP::new();
    injector
        .when_called(injectorpp::func!(fn (Path::exists)(&Path) -> bool))
        .will_execute_raw(injectorpp::func!(fn (fake_path_exists)(&Path) -> bool));

    let test_path = "/path/that/does/not/exist";
    let result = Path::new(test_path).exists();

    assert_eq!(result, true);
}
```

Below is an example of using closure:

```rust
#[test]
fn test_will_execute_raw_when_fake_no_return_function_use_closure_should_success() {
    static CALL_COUNT_CLOSURE: AtomicU32 = AtomicU32::new(0);

    let fake_closure = || {
        CALL_COUNT_CLOSURE.fetch_add(1, Ordering::SeqCst);
    };

    let mut injector = InjectorPP::new();
    injector
        .when_called(injectorpp::func!(fn (func_no_return)()))
        .will_execute_raw(injectorpp::closure!(fake_closure, fn()));

    func_no_return();

    assert_eq!(CALL_COUNT_CLOSURE.load(Ordering::SeqCst), 1);
}
```

## `Fake async functions`

To fake async functions, `when_called_async` and `will_return_async` are needed.

Below is an example to fake simple async functions:

```rust
async fn simple_async_func_u32_add_one(x: u32) -> u32 {
    x + 1
}

async fn simple_async_func_u32_add_two(x: u32) -> u32 {
    x + 2
}

async fn simple_async_func_bool(x: bool) -> bool {
    x
}

#[tokio::test]
async fn test_simple_async_func_should_success() {
    let mut injector = InjectorPP::new();

    injector
        .when_called_async(injectorpp::async_func!(
            simple_async_func_u32_add_one(u32::default()),
            u32
        ))
        .will_return_async(injectorpp::async_return!(123, u32));

    let x = simple_async_func_u32_add_one(1).await;
    assert_eq!(x, 123);

    // simple_async_func_u32_add_two should not be affected
    let x = simple_async_func_u32_add_two(1).await;
    assert_eq!(x, 3);

    injector
        .when_called_async(injectorpp::async_func!(
            simple_async_func_u32_add_two(u32::default()),
            u32
        ))
        .will_return_async(injectorpp::async_return!(678, u32));

    // Now because it's faked the return value should be changed
    let x = simple_async_func_u32_add_two(1).await;
    assert_eq!(x, 678);

    // simple_async_func_bool should not be affected
    let y = simple_async_func_bool(true).await;
    assert_eq!(y, true);

    injector
        .when_called_async(injectorpp::async_func!(
            simple_async_func_bool(bool::default()),
            bool
        ))
        .will_return_async(injectorpp::async_return!(false, bool));

    // Now because it's faked the return value should be false
    let y = simple_async_func_bool(true).await;
    assert_eq!(y, false);
}
```

Below is an example to fake a complex struct method:

```rust
struct HttpClientTest {
    pub url: String,
}

impl HttpClientTest {
    pub async fn get(&self) -> String {
        format!("GET {}", self.url)
    }

    pub async fn post(&self, payload: &str) -> String {
        format!("POST {} to {}", payload, self.url)
    }
}

#[tokio::test]
async fn test_complex_struct_async_func_without_param_should_success() {
    {
        // This is a temporary instance that is needed for async function fake.
        // Parameter does not matter.
        let temp_client = HttpClientTest {
            url: String::default(),
        };

        let mut injector = InjectorPP::new();
        injector
            .when_called_async(injectorpp::async_func!(temp_client.get(), String))
            .will_return_async(injectorpp::async_return!(
                "Fake GET response".to_string(),
                String
            ));

        // Now the real client will be used and its behavior should be faked
        let real_client = HttpClientTest {
            url: "https://test.com".to_string(),
        };

        let result = real_client.get().await;
        assert_eq!(result, "Fake GET response".to_string());
    }

    let real_client = HttpClientTest {
        url: "https://test.com".to_string(),
    };

    // The original function should be called as the injector is out of scope
    let result = real_client.get().await;
    assert_eq!(result, "GET https://test.com".to_string());
}
```

## `Fake system functions`

Traditionally, system functions could cause the code non-unit testable immediately. It's also one of the test challenges in the projects rely on low level system apis. Now with injectorpp, system function can be easily faked. Below is an example:

```rust
use std::ffi::CString;
use std::os::raw::{c_char, c_int, c_uint};

use injectorpp::interface::injector::*;

extern "C" {
    fn shm_open(name: *const c_char, oflag: c_int, mode: c_uint) -> c_int;
}

#[test]
fn test_fake_shm_open_should_return_fixed_fd() {
    // Fake shm_open to always return file descriptor 32
    let mut injector = InjectorPP::new();
    injector
        .when_called(injectorpp::func!(
            unsafe{} extern "C" fn (shm_open)(*const c_char, c_int, c_uint) -> c_int
        ))
        .will_execute(injectorpp::fake!(
            func_type: unsafe extern "C" fn(_name: *const c_char, _oflag: c_int, _mode: c_uint) -> c_int,
            returns: 32
        ));

    let name = CString::new("/myshm").unwrap();
    let fd = unsafe { shm_open(name.as_ptr(), 0, 0o600) };
    assert_eq!(fd, 32);
}
```

## `Fake Azure SDK client library`

Mocking Azure SDK client library related to http or https request was tough. But by using injectorpp it's simple. Below is an example:

```rust
#[tokio::test]
async fn test_azure_http_client_always_return_200() {
    // Create a temporary client + request to capture the method pointer
    let temp_client = new_http_client();
    let mut temp_req = Request::new(Url::parse("https://temp/").unwrap(), Method::Get);

    // Setup the fake
    let mut injector = InjectorPP::new();
    injector
        .when_called_async(injectorpp::async_func!(
            temp_client.execute_request(&mut temp_req),
            std::result::Result<RawResponse, Error>
        ))
        .will_return_async(injectorpp::async_return!(
            // always return an Ok(RawResponse) with status 200
            Ok(RawResponse::from_bytes(StatusCode::Ok, Headers::new(), vec![])),
            std::result::Result<RawResponse, Error>
        ));

    // Run the real code under test
    let client = new_http_client();
    let url = Url::parse("https://nonexistsitetest").unwrap();
    let mut request = Request::new(url, Method::Get);

    let response = client.execute_request(&mut request).await.unwrap();
    assert_eq!(response.status(), 200);
}
```

## `Unsafe API`

`when_called_unchecked` and `will_execute_raw_unchecked` are the unsafe versions of `when_called` and `will_execute_raw`. They allow you to bypass type check but you need to ensure the safety yourself.

```rust
pub fn fake_path_exists(_path: &Path) -> bool {
    println!("fake_path_exists executed.");
    true
}

#[test]
fn test_will_execute_raw_unchecked_when_fake_file_dependency_should_success() {
    let mut injector = InjectorPP::new();

    unsafe {
        injector
            .when_called_unchecked(injectorpp::func_unchecked!(Path::exists))
            .will_execute_raw_unchecked(injectorpp::func_unchecked!(fake_path_exists));
    }

    let test_path = "/path/that/does/not/exist";
    let result = Path::new(test_path).exists();

    assert_eq!(result, true);
}
```

Similarly, `when_called_async_unchecked` and `will_return_async_unchecked` are the unsafe versions for async functions.

```rust
async fn simple_async_func_u32_add_one(x: u32) -> u32 {
    x + 1
}

async fn simple_async_func_u32_add_two(x: u32) -> u32 {
    x + 2
}

async fn simple_async_func_bool(x: bool) -> bool {
    x
}

#[tokio::test]
async fn test_simple_async_func_unchecked_should_success() {
    let mut injector = InjectorPP::new();

    unsafe {
        injector
            .when_called_async_unchecked(injectorpp::async_func_unchecked!(
                simple_async_func_u32_add_one(u32::default())
            ))
            .will_return_async_unchecked(injectorpp::async_return_unchecked!(123, u32));
    }

    let x = simple_async_func_u32_add_one(1).await;
    assert_eq!(x, 123);

    // simple_async_func_u32_add_two should not be affected
    let x = simple_async_func_u32_add_two(1).await;
    assert_eq!(x, 3);

    unsafe {
        injector
            .when_called_async_unchecked(injectorpp::async_func_unchecked!(
                simple_async_func_u32_add_two(u32::default())
            ))
            .will_return_async_unchecked(injectorpp::async_return_unchecked!(678, u32));
    }

    // Now because it's faked the return value should be changed
    let x = simple_async_func_u32_add_two(1).await;
    assert_eq!(x, 678);

    // simple_async_func_bool should not be affected
    let y = simple_async_func_bool(true).await;
    assert_eq!(y, true);

    unsafe {
        injector
            .when_called_async_unchecked(injectorpp::async_func_unchecked!(simple_async_func_bool(
                bool::default()
            )))
            .will_return_async_unchecked(injectorpp::async_return_unchecked!(false, bool));
    }

    // Now because it's faked the return value should be false
    let y = simple_async_func_bool(true).await;
    assert_eq!(y, false);
}
```

# Contributing

This project welcomes contributions and suggestions. Please see the [CONTRIBUTING.md](CONTRIBUTING.md)

# Trademarks

This project may contain trademarks or logos for projects, products, or services. Authorized use of Microsoft
trademarks or logos is subject to and must follow
[Microsoft's Trademark & Brand Guidelines](https://www.microsoft.com/legal/intellectualproperty/trademarks/usage/general).
Use of Microsoft trademarks or logos in modified versions of this project must not cause confusion or imply Microsoft sponsorship.
Any use of third-party trademarks or logos are subject to those third-party's policies.
