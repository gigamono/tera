### SECURITY

**How is this different from Deno**

`Deno` is a general Typescript and JavaScript runtime with a permission system that is currently only available via the CLI.
`secure-runtime`, on the other hand, is designed from the ground up for secure execution of JavaScript code in multi-tenant serverless environment.
This means we have to be careful about the interfaces that get exposed to the user by reducing the number of  making capabilities-based security system a core part of the interface.

##

### THE RUNTIME

The `runtime::Runtime` is responsible for initializing the JavaScript runtime with necessary state and options.

##

### THE EXTENSIONS

##

### THE CAPABILITIES

##

### THE LOADERS
