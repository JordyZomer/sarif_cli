# sarif_cli
A SARIF viewer for the command-line.

Could be useful for CodeQL or Semgrep output.

## Example

```sh
% ./target/debug/sarif_cli tests/test.sarif
tests/test.c:6:9
================================
5: int main(int argc, char *argv[]) {
6: 	printf(argv[1], argv[2]);
------------------^
ALERT: "User-input is used as a format specifier to printf"
-------------------
7: 	return 0;
8: }
```
