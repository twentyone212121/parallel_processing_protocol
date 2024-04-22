# parallel_processing_protocol

This repository includes a server and client written in Rust that communicate with each other using custom application-level protocol on top of TCP. It is capable of transferring any square matrix of signed integers along with a number of threads to process the matrix.

There is also a client written in Python that communicates in the same way with the server.

Protocol UML description:

![UML diagram of client's and server's communication](https://www.planttext.com/api/plantuml/svg/hLEzReCm4DxlAKvqALB30p1KeK5T6a5KM3gwn25Mt3EoJTMyVSrFcn18wD3Dkll-V3ulZMNQrXyIe787KOYAaSML59milAilz3AkFxdsy6ujohrzaZKVGBlseEHrTCptchEjD-pbp1nmerN1ZH5KzY3Z0QSjEaE0JX4NYyTUFCIiIObI6Cl9aG1wXn6aK4Iyi4BHkSn5e10ZvrqI6bKSk8KW2CuLH-Zxx2tnuQBS2dUb7e7PaJ1QlSphnL7kOymimvtIMFqEuw79lnDbhLbxXPkM9v4_UMPvvFT3EXdEN6gAqhfDqQw-R3YsdKsxITS-ld9JIyTJKjuEcgHk58cosFIEdVhLEDCRqGvIjuR_62nEtLey2sBoHnkMRi_TV_i6)
