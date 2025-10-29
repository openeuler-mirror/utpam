# utpam

#### Introduction
utpam is an authentication framework for Linux systems. It allows system administrators to define authentication mechanisms for different services and combine multiple authentication methods as needed. utpam provides a unified interface for handling authentication, simplifying the process for applications to verify user identities.

#### Background
utpam is a project aimed at re-implementing traditional PAM using Rust. The original PAM tools are written in C and are frequently called in the system to handle authentication tasks. However, due to limitations of the C language, traditional PAM may suffer from memory leaks and other issues over long-term operation, potentially leading to security vulnerabilities or instability.
Therefore, the utpam project was created. It leverages Rust's powerful features and modern programming practices to overcome some inherent problems of traditional PAM in memory management and security. Rust enforces memory safety rules and can detect common errors such as buffer overflows and null pointer dereferences at compile time, effectively preventing potential security risks.

#### Software Architecture
- Modular design: Each authentication method is implemented as a separate module, supporting custom extensions.
- Core library: Responsible for module loading, configuration parsing, and authentication flow.
- Compatibility: Compatible with existing PAM configurations for easy migration and integration.
- Multi-language support: Core implemented in Rust, with C-compatible interfaces.

#### Main Features
- Unified authentication interface: utpam provides a unified API for different applications, allowing them to perform authentication operations by calling the same functions without worrying about the underlying mechanisms.
- Flexibility and extensibility: Administrators can easily change authentication policies for applications via configuration files, without recompiling or modifying application code.
- Modular design: utpam consists of multiple independent modules, each handling a specific type of authentication task, such as user authentication, account management, session management, and password management. This modular design allows for customized authentication flows by selecting and combining different modules as needed.

#### Installation

1. Install Rust (recommended via [rustup](https://rustup.rs/)):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. Clone the repository:
   ```bash
   git clone https://gitee.com/yourname/utpam.git
   ```
3. Enter the project directory and build:
   ```bash
   cd utpam
   cargo build --release
   # Optionally install to system directory
   sudo cp target/release/libutpam.so /usr/local/lib/
   ```

#### Instructions

1. Edit the configuration file (e.g., `/etc/utpam.conf`) to specify authentication modules and order.
2. Integrate utpam with your application or system service by linking to the utpam library.
3. Use provided command-line tools or APIs to perform authentication.
4. Refer to test cases in `libutpam/tests/` for usage examples.

#### Contribution

1. Fork the repository
2. Create a `Feat_xxx` branch
3. Commit your code
4. Create a Pull Request

## License
utpam is licensed under [GPL-2.0-or-later](LICENSE)
