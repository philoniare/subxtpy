# Keypair

A `Keypair` used for signing transactions.

This class wraps an `sr25519` keypair and provides methods to create a keypair from a secret key.

## Methods

### `from_secret_key(secret_key: str) -> Keypair`

Create a new `Keypair` from a secret key in hexadecimal format.

**Parameters:**

- `secret_key` (str): A 64-character hexadecimal string representing the secret key.

**Returns:**

- `Keypair`: A new `Keypair` instance.

**Raises:**

- `ValueError`: If the secret key is not 64 hex characters long or invalid.

**Example:**

```python
from subxtpy import Keypair

secret_key = "your_64_character_hex_secret_key"
keypair = Keypair.from_secret_key(secret_key)
```