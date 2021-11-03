### SECURITY

At the core of `secure-runtime` is an `isolate` which represents a isolated JavaScript runtime that does not share memory with other isolates running in the same process. Shared memory is a recipe for exploits and vulnerabilities and this architecture helps prevent that.

`secure-runtime` creates a permission system on top of this that prevents running modules from accessing resources the user has not authorized. `secure-runtime` provides a secure implementation of common extensions like `fs`.


#### Why We Want a Capability-based Secure Runtime

We are in an age where we combine multiple software of different origins and makers to create our own solution. Reuse is a great but it comes with security issues as evidenced by the number of CVEs related to [exploits introduced into open-source packages every year](). We let third party packages run on our behalf but we grant them too much power that many of them don't need. This has to change and it is the reason why `secure-runtime` exists.

`secure-runtime` allows only the capabilities the user has allowed for any running module. If a running module tries to access a resource it not authorized to, an exception is thrown.

There are similar initiatives like [WASI] and [Capsicum].

#### How is This Different from Deno?

`Deno` is a general Typescript and JavaScript runtime with a permission system that is currently only available via the CLI.
On the other hand, `deno-core`, the project `Deno` is based on, provide the necessary primitives for buidling a similar system for non-interactive scenarios.
So `secure-runtime` is based on `deno-core` and it is designed from the start for secure execution of JavaScript code in a multi-tenant environment like servers.

Currently, there is a lot going on in the `Deno` project because its goal is to be general. For example, host functions (that sometimes expose system calls) are implemented without any permissions guarding their use.

`Deno` also supports TypeScript, `secure-runtime` doesn't.

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
