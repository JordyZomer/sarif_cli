# sarif_cli

A SARIF viewer for the command-line.

Because numerous static analysis tools, such as #CodeQL and #semgrep, use the SARIF format.
I decided to create a command-line application to display the alerts and learn some more Rust in the process.

## Example

```bash
% ./target/debug/sarif_cli
Usage: ./target/debug/sarif_cli <file_path> <source_dir>
% ./target/debug/sarif_cli tests/flexarray.sarif ~/kernels/linux-5.13.12
/Users/jordy/kernels/linux-5.13.12/block/blk-map.c:32:2
================================
21: static struct bio_map_data *bio_alloc_map_data(struct iov_iter *data,
22: 					       gfp_t gfp_mask)
23: {
24: 	struct bio_map_data *bmd;
25:
26: 	if (data->nr_segs > UIO_MAXIOV)
27: 		return NULL;
28:
29: 	bmd = kmalloc(struct_size(bmd, iov, data->nr_segs), gfp_mask);
30: 	if (!bmd)
31: 		return NULL;
32: 	memcpy(bmd->iov, data->iov, sizeof(struct iovec) * data->nr_segs);
------^
ALERT: "This memcpy has a flexible-array-member as a destination: [call to memcpy](1)"
-------
33: 	bmd->iter = *data;
34: 	bmd->iter.iov = bmd->iov;
35: 	return bmd;
36: }
```
