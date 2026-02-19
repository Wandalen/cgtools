# How to run a Rust/WebAssembly example

## 1. Install Rust

First, you need to install `Rust`. Go to [https://rust-lang.org/tools/install/](https://rust-lang.org/tools/install/)
and follow the instructions for your platform.

---

## 2. Install the wasm32 target

After that, you need to install the `wasm32` target. Open a new terminal.

* **Windows**: open the Start menu, find and launch PowerShell or the Terminal app.
* **macOS**: press `Cmd` + `Space`, type "Terminal" in the search bar, and press `Enter`.
* **Linux**: this depends on the distro, so check how to open a terminal for your specific one.

After opening the terminal, paste the following command and press `Enter`:

```sh
rustup target add wasm32-unknown-unknown
```

---

## 3. Download cgtools

Go to this link: [https://github.com/Wandalen/cgtools/archive/refs/heads/master.zip](https://github.com/Wandalen/cgtools/archive/refs/heads/master.zip).
It should start downloading the `cgtools` repository.
(*If you know how to use Git, you can just clone it instead.*)

Once the download is finished, unzip the contents of the archive into a convenient directory.

---

## 4. Install trunk

The next thing we need is `trunk`. There is an installation guide on the [official site](https://trunkrs.dev/),
but it suggests installing via `cargo install trunk`, which compiles trunk from source and can take a long time.
To save time, I recommend downloading a prebuilt binary instead.

Go to the [GitHub releases page](https://github.com/trunk-rs/trunk/releases)
and download the appropriate version for your system.

* **Windows**: your only option is `trunk-x86_64-pc-windows-msvc`.
* **macOS**: if you have an Apple M-series processor, download `trunk-aarch64-apple-darwin.tar.gz`,
  otherwise use `trunk-x86_64-apple-darwin.tar.gz`.
* **Linux**: most likely you will need `trunk-x86_64-unknown-linux-gnu.tar.gz`, or
  `trunk-aarch64-unknown-linux-gnu.tar.gz` if you are on ARM.

Next, go to your Rust installation directory. By default, this is:

* `$HOME/.cargo/bin` on Linux/macOS
* `%USERPROFILE%\.cargo\bin` on Windows

(if you didn't change it during setup).

Extract the downloaded `trunk` binary into this directory.
If you followed the instructions from the official `trunk` site, you can skip this step.

To verify that everything is working correctly, open a terminal and run:

```sh
trunk --version
```

If everything is set up properly, you should see something like:

```text
trunk 0.X.Y
```

The exact version number will differ — that's fine as long as the command succeeds.

---

## 5. Run the example

When everything is set up, navigate to the `cgtools` folder that you downloaded and unzipped earlier.
Go into `examples/minwebgl` and find the folder of the example you want to run.
You need to open a terminal in that folder.

* **Windows**: right-click the folder and select "Open in Terminal", or hold `Shift`, right-click,
  and choose "Open PowerShell window here".
* **macOS**: right-click the folder in Finder, select "Services", then choose
  "New Terminal at Folder" or "New Terminal Tab at Folder".
  *Note: if this option doesn't appear, enable it via System Settings → Keyboard → Keyboard Shortcuts →
  Services → Files and Folders.*
* **Linux**: this depends on the distro, so check how to do it for your specific one.

As a result, you should see a terminal window with the path pointing to the example folder. For example:

```text
PS C:\Users\Admin\Documents\repos\cgtools\examples\minwebgl\some_example>
```

Your path will look different, but should end with `examples\minwebgl\` followed by the example name.

Now run:

```sh
trunk serve --release
```

After a short while, you should see output similar to this:

```text
2026-01-28T13:34:30.471030Z  INFO applying new distribution
2026-01-28T13:34:30.485404Z  INFO success
2026-01-28T13:34:30.485686Z  INFO serving static assets at -> /
2026-01-28T13:34:30.489735Z  INFO server listening at:
2026-01-28T13:34:30.489893Z  INFO     http://127.0.0.1:8080/
2026-01-28T13:34:30.489994Z  INFO     http://[::1]:8080/
2026-01-28T13:34:30.493532Z  INFO     http://localhost.:8080/
```

This means everything has been built successfully.
Now open your browser and go to `http://127.0.0.1:8080/`.
