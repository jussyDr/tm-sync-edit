# Model Checking

[TLA<sup>+</sup>](https://lamport.azurewebsites.net/tla/tla.html) models used to verify the correctness of the synchronization algorithms used in SyncEdit.

| Model | Specifies |
| --- | --- |
| Blocks | Synchronization of standard (non ghost or free) blocks |
| FreeObjects | Synchronization of ghost blocks, free blocks and items |

## Usage

Since the models are written in PlusCal, they need to first be translated into TLA<sup>+</sup> after each modification. Then, the models can be checked using the TLC model checker. This can for instance be done using the [TLA<sup>+</sup> VS Code extension](https://marketplace.visualstudio.com/items?itemName=alygin.vscode-tlaplus). 
