
## std::crypto::fri::helper
| Procedure | Description |
| ----------- | ------------- |
| generate_fri_parameters | Compute the number of FRI layers given log2 of the size of LDE domain. It also computes the<br />LDE domain generator and, from it, the trace generator and store these for later use.<br /><br />Input: [...]<br />Output: [num_fri_layers, ...]<br />Cycles: 52<br /> |
| load_fri_layer_commitments | Get FRI layer commitments and reseed with them in order to draw folding challenges i.e. alphas.<br /><br />Input: [ptr_layer, num_layers, ...]<br />Output: [...]<br />Cycles: 21 + 83 * num_fri_layers<br /> |
| load_and_verify_remainder | Load the remainder polynomial from the advice provider and check that its hash corresponds<br />to its commitment and reseed with the latter.<br />Load the remainder code word, i.e. the NTT of the remainder polynomial, and use its hash, together,<br />with the hash of the remainder polynomial in order to generate the Fiat-Shamir challenge `tau` for<br />the `verify_remainder_xx` procedure.<br /><br />Input: [...]<br />Output: [...]<br />Cycles:<br />1- Remainder of size 32: 1633<br />2- Remainder of size 64: 3109<br /> |
