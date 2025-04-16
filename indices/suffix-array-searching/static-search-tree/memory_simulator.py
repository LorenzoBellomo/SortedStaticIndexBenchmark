import random
import sys

CL_SIZE = 64
CACHE_SIZE = 2 ** 22
U32_SIZE = 4
CACHE_ASSOCIATIVITY = 1

class Memory:
    def __init__(self, ):
        self.cache = {}
        self.mm_accesses = 0

    def request(self, addr: int):
        index = (addr >> 7) & (CL_SIZE - 1)
        tag = addr // CACHE_SIZE
        if index not in self.cache:
            self.mm_accesses += 1
        elif index in self.cache and self.cache[index] != tag:
            self.mm_accesses += 1
        self.cache[index] = tag

class BatchedBinsearch:
    def __init__(self, num_queries: int, batch_size: int, array_size: int):
        self.num_queries = num_queries
        self.memory = Memory()
        self.array_size = array_size
        self.batch_size = batch_size

    def simulate_batch(self):
        bases = [0] * self.batch_size
        len = self.array_size
        while len > 1:
            mid = len // 2
            len -= mid
            for i in range(self.batch_size):
                self.memory.request(bases[i] + mid * U32_SIZE)
                to_jump = random.randint(0, 1)
                bases[i] += to_jump * mid
        # print(bases)

    def simulate(self):
        for i in range(self.num_queries // self.batch_size):
            self.simulate_batch()
        print("Binsearch memory accesses: ", self.memory.mm_accesses)

class BatchedEytzinger:
    def __init__(self, num_queries: int, batch_size: int, array_size: int):
        self.num_queries = num_queries
        self.memory = Memory()
        self.array_size = array_size
        self.batch_size = batch_size

    def simulate_batch(self):
        positions = [1] * self.batch_size
        num_iters = self.array_size.bit_length() + 1
        for i in range(num_iters):
            for j in range(self.batch_size):
                jump_to = random.randint(0, 1)
                self.memory.request(positions[j] * U32_SIZE)
                positions[j] = 2 * positions[j] + jump_to

    def simulate(self):
        for i in range(self.num_queries // self.batch_size):
            self.simulate_batch()
        print("Eytzinger memory accesses: ", self.memory.mm_accesses)

if __name__ == "__main__":
    batch_size = 1
    array_size = int(1.17 ** 110)
    print("Array size: ", array_size * U32_SIZE, "Cache size", CACHE_SIZE)
    num_batches = 10000
    while batch_size < 256:
        print("Batch size: ", batch_size, "Array size:", array_size)
        binsearch = BatchedBinsearch(batch_size * num_batches, batch_size, array_size)
        binsearch.simulate()
        eytzinger = BatchedEytzinger(batch_size * num_batches, batch_size, array_size)
        eytzinger.simulate()
        batch_size *= 2
