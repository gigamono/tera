### SECURITY

At the core of `secure-runtime` is an `isolate`, a JavaScript vm that does not share memory with other vm instances running in the same process. Shared memory is a recipe for exploits and vulnerabilities; this architecture helps prevent that.

#### Why We Want a Capability-based Secure Runtime

We are in a time where we combine multiple software of different origins and makers to create our own solution. Reuse is great but it comes with security issues as evidenced by the number of CVEs related to [exploits introduced into open-source packages every year](). We let third party packages run on our behalf but we grant them too much power that many of them don't need. This has to change and it is the reason why `secure-runtime` exists.

`secure-runtime` allows only the capabilities the user has allowed for any running module. If a running module tries to access a resource it is not authorized to, an exception is thrown.

There are similar initiatives like [WASI] and [Capsicum].

#### How is This Different from Deno?

`Deno` is indeed similar to `secure-runtime`; both are based on `deno_core`. They both introduce runtime permissions and capability-based security, however `secure-runtime` is designed to have a more granular permission system that is extendable by the developer.

##

### THE RUNTIME

The `runtime::Runtime` is responsible for initializing the JavaScript runtime with necessary state and options.

##

### THE EXTENSIONS

A JavaScript runtime by itself is boring because it is a sandboxed vm without access to [system resources]. Typical applications would need access to files, databases, etc. This is achieved with extensions.

##

### THE CAPABILITIES

A capability-based system requires non-forgeable tokens to ensure secure access to resources.

##

### THE LOADERS
