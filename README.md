# Toy payment engine

## Further improvements

1. It is assumed that the number of customers/transactions is reasonable, i.e. they can be stored in
   the memory. In a production system, a real database should replace the hash maps
1. Since nothing specific is done with the errors (they are just printed to the console), all the
   errors are a simple string. In a production system, different error type (enum) should be
   considered
1. Integration tests are missing because of lack of time but they are crucial in a production system
1. The loading of data (from the csv file) and the processing of the transactions could be
   parallelized. Since parallelizing increases the complexity of the code, it is important to
   profile the application (with some real examples) to identify the bottlenecks before trying to
   optimize the application
1. The `buffer_size` allows to parametrize how much of the file should be loaded at the same time.
   This parameter is a trade-off between high performance (large `buffer_size`) and reduced memory
   footprint.
