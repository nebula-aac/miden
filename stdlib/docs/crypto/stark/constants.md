
## std::crypto::stark::constants
| Procedure | Description |
| ----------- | ------------- |
| lde_size_ptr | Address to store details about the lde size.<br /><br />Memory is `[lde_size, log(lde_size), lde_g, 0]`<br /> |
| z_ptr | Address for the point `z` and its exponentiation `z^N` where `N=trace_len`.<br /><br />Memory is `[(z_1, z_0)^n, z_1, z_0]`<br /> |
| c_ptr | Returns the pointer to the capacity word of the random coin.<br /><br />Note: The random coin is implemented using a hash function, this returns the<br />capacity portion of the RPO.<br /> |
| r1_ptr | Returns the pointer to the first rate word of the random coin.<br /><br />Note: The random coin is implemented using a hash function, this returns the<br />first rate word of the RPO.<br /> |
| r2_ptr | Returns the pointer to the second rate word of the random coin.<br /><br />Note: The random coin is implemented using a hash function, this returns the<br />second rate word of the RPO.<br /> |
| tmp1 | Address to store details to compute deep query denominators.<br /><br />Memory is `[gz1, gz0, z_1, z_0]`<br /> |
