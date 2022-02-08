# sarif_cli
A SARIF viewer for the command-line.

Could be useful for CodeQL or Semgrep output.

At the moment this is WIP and only supports C, it may not work as expected yet.

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

## Todo

- Test this on real SARIF output
- Visualize data-flow
- Add support for more languages
- Clean up code
