# servirtium-rust
A Rust implementation of Servirtium, a library that helps test interactions
with APIs. 

## How it works
Servirtium is a server that serves as a man-in-the-middle: it processes
incoming requests, forwards them to a destination API and writes the
response into a Markdown file with a special format that is common across all
of the implementations of the library. Later these Markdown files are used to
replay the interactions that were recorded before, allowing to test
interactions without making real API calls.

## Prerequisites
The library is written in the Rust programming language, so to use or compile
the library you need **Cargo**. You can download it on the
[official site](https://www.rust-lang.org/).

## Building
To build the library, type the following into the command line in the project
root directory:
```shell script
$ cargo build
```

## Usage
The library isn't published to **crates.io** yet, so you can't refer to the
package the ordinary way. You need to specify a git dependency instead in the 
`Cargo.toml`. It is better to refer to the library as a development dependency,
because it is only useful in tests:
```toml
[dev-dependencies]
servirtium = { git = "https://github.com/servirtium/servirtium-rust" }
```

After specifying the dependency, you can place one of the following attributes
on a test function to start the Servirtium server and configure it to serve
requests in record or playback mode.
```rust
use servirtium::{
    servirtium_record_test,
    servirtium_playback_test
};

// record mode, write the results into the specified markdown. Forward API
// requests to the specified url.
#[servirtium_record_test("path_to_markdown.md", "https://exampleapi.org")]
fn record_test() {
    // make some calls to localhost:61417 ...
}

// playback mode. Don't forward API requests to the destination API, but
// replay the responses according to the data in the specified markdown
#[servirtium_playback_test("path_to_markdown.md", "https://exampleapi.org")]
fn playback_test() {
    // make some calls to localhost:61417 ...
}
```

You can also pass a configuration function instead of a domain name to the
attribute to allow more fine-grained configuration of the Servirtium server:
```rust
use servirtium::{ServirtiumConfiguration, servirtium_record_test};

fn configure(config: &mut ServirtiumConfiguration) {
    config.set_domain_name("https://exampleapi.org");
    config.set_fail_if_markdown_changed(true);

    config.add_record_response_mutations(|builder| {
        builder.remove_headers(vec!["set-cookie", "date"])
    });

    config.add_playback_response_mutations(|builder| {
        builder.add_header("date", "Sun, 02 Aug 2020 09:53:31 GMT")
    });
}

// call the configure function before executing the test code
#[servirtium_record_test("path_to_markdown.md", configure)]
fn playback_test() {
    // make some calls to localhost:61417 ...
}
```

When the tests are run, a single Servirtium server instance is run in a
separate thread (in-process) and starts listening on port `61417`.

In record mode all requests are forwarded to the destination API and the
responses are written in the markdown file specified in the attribute.

In playback mode the Servirtium server replays all the interactions occurred in
record mode without accessing the destination API.

## Example
You can find a sample project that uses the library in the following
repository: 
[demo-rust-climate-tck](https://github.com/servirtium/demo-rust-climate-tck)

## License
Licensed under MIT License ([LICENSE](LICENSE) or
http://opensource.org/licenses/MIT)