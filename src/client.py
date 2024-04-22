import socket
import sys
import random
import struct

class Matrix:
    def __init__(self, n):
        min_val = 0
        max_val = 9
        diff = max_val - min_val

        outer = []
        for _ in range(n):
            row = [min_val + ((random.randint(0, diff) + diff) % diff) 
                   for _ in range(n)]
            outer.append(row)

        self.data = outer

    def serialized_size(self):
        return 4 + (self.get_dim() * self.get_dim()) * 4

    def serialize(self):
        matrix_dim = len(self.data)

        serialized = bytearray()
        serialized.extend(struct.pack('>I', matrix_dim))
        for row in self.data:
            for val in row:
                serialized.extend(struct.pack('>i', val))

        return serialized

    def deserialize(self, data):
        matrix_dim = struct.unpack_from('>I', data)[0]
        elem_size = 4

        offset = 4
        matrix_data = []
        for _ in range(matrix_dim):
            row = struct.unpack_from('>' + 'i' * matrix_dim, data, offset)
            matrix_data.append(list(row))
            offset += elem_size * matrix_dim

        self.data = matrix_data

    def get_dim(self):
        return len(self.data)

def main():
    args = sys.argv
    if len(args) != 4:
        print(f"Usage: {args[0]} thread_num matrix_dim to_print_matrix")
        return

    thread_num = int(args[1])
    matrix_dim = int(args[2])
    to_print = bool(int(args[3]))

    print("Constructing matrix...")
    matrix = Matrix(matrix_dim)
    if to_print:
        print(f"Hello, I am client with matrix:\n{matrix.data}")
    else:
        print("Hello, I am client with matrix")

    serialized = matrix.serialize()

    stream = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    stream.connect(("127.0.0.1", 7878))

    syn = "SYN".encode()
    stream.sendall(syn)
    print("Sent: SYN")

    answer_bytes = stream.recv(3)
    print(f"Received: {answer_bytes.decode()}")

    dat = "DAT".encode()
    thread_num_bytes = thread_num.to_bytes(4, byteorder='big')
    data_request = dat + thread_num_bytes + serialized
    stream.sendall(data_request)
    print("Sent: DAT")

    answer_bytes = stream.recv(3)
    print(f"Received: {answer_bytes.decode()}")

    sta = "STA".encode()
    stream.sendall(sta)
    print("Sent: STA")

    answer_bytes = stream.recv(3)
    print(f"Received: {answer_bytes.decode()}")

    pol = "POL".encode()
    stream.sendall(pol)
    print("Sent: POL")

    answer_bytes = stream.recv(3)
    print(f"Received: {answer_bytes.decode()}")

    expected_size = matrix.serialized_size()
    matrix_buf = bytearray(expected_size)
    while True:
        pol = "POL".encode()
        stream.sendall(pol)
        print("Sent: POL")

        answer_bytes = stream.recv(3)
        print(f"Received: {answer_bytes.decode()}")

        if answer_bytes.decode() == "DON":
            stream.recv_into(matrix_buf)
            matrix.deserialize(matrix_buf)
            if matrix.data is not None:
                if to_print:
                    print(f"Received matrix: {matrix.data}")
                else:
                    print("Received matrix")
            else:
                print("Received matrix is incorrect")
            break

    stream.close()

if __name__ == "__main__":
    main()

