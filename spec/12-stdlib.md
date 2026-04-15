# 12. Standard Library

This chapter specifies the Monel standard library. The standard library is provided as modules. No types or functions are language primitives (except the syntax for literals, control flow, and the effect system, which are specified in earlier chapters). All standard library modules are self-hosting: they are written in Monel with their own contracts, subject to the same parity verification as user code.

---

## 12.1 Core Types

The following types form the standard prelude and are automatically in scope for every module (see Section 7.5.4).

### 12.1.1 Primitive Types

| Type | Module | Description | Size |
|------|--------|-------------|------|
| `Int` | `std/num` | Platform-width signed integer | 64-bit |
| `Int8` | `std/num` | 8-bit signed integer | 1 byte |
| `Int16` | `std/num` | 16-bit signed integer | 2 bytes |
| `Int32` | `std/num` | 32-bit signed integer | 4 bytes |
| `Int64` | `std/num` | 64-bit signed integer | 8 bytes |
| `Float` | `std/num` | 64-bit IEEE 754 floating point | 8 bytes |
| `Float32` | `std/num` | 32-bit IEEE 754 floating point | 4 bytes |
| `Bool` | `std/bool` | Boolean (`true` / `false`) | 1 byte |
| `Byte` | `std/num` | Unsigned 8-bit integer | 1 byte |
| `Char` | `std/char` | Unicode scalar value (32-bit) | 4 bytes |
| `String` | `std/string` | UTF-8 encoded string (owned) | 3 words |
| `Unit` | `std/unit` | The unit type, single value `()` | 0 bytes |
| `Never` | `std/never` | The never type (uninhabited) | 0 bytes |

**Unsigned integer types** (`UInt`, `UInt8`, `UInt16`, `UInt32`, `UInt64`, `ULong`) are defined in `std/num` and re-exported at the top level.

### 12.1.2 Composite Types

| Type | Module | Description |
|------|--------|-------------|
| `Array<T, N>` | `std/array` | Fixed-size array, stack-allocated |
| `Vec<T>` | `std/vec` | Growable array, heap-allocated |
| `Map<K, V>` | `std/collections` | Hash map |
| `Set<T>` | `std/collections` | Hash set |
| `Option<T>` | `std/option` | Optional value (`Some(T)` or `None`) |
| `Result<T, E>` | `std/result` | Success or error (`Ok(T)` or `Err(E)`) |
| `Tuple<...>` | built-in | Heterogeneous fixed-size product type |
| `Range<T>` | `std/range` | Range of values (`start..end`) |

### 12.1.3 Pointer Types

| Type | Module | Description |
|------|--------|-------------|
| `Ptr<T>` | `std/mem` | Immutable raw pointer |
| `MutPtr<T>` | `std/mem` | Mutable raw pointer |
| `Box<T>` | `std/mem` | Heap-allocated owned pointer |
| `Rc<T>` | `std/mem` | Reference-counted pointer |
| `Arc<T>` | `std/sync` | Atomically reference-counted pointer |

---

## 12.2 `std/io`: Input/Output

Provides fundamental I/O traits and types for reading and writing byte streams.

### 12.2.1 Core Traits

```
trait Read
  doc: "a source of bytes that can be read from"
  fn read(self: mut Self, buf: mut Array<Byte>) -> Result<Int, IoError> with Fs.read
  fn read_exact(self: mut Self, buf: mut Array<Byte>) -> Result<Unit, IoError> with Fs.read
  fn read_all(self: mut Self) -> Result<Vec<Byte>, IoError> with Fs.read
  fn read_to_string(self: mut Self) -> Result<String, IoError> with Fs.read
```

```
trait Write
  doc: "a destination for bytes that can be written to"
  fn write(self: mut Self, buf: Array<Byte>) -> Result<Int, IoError> with Fs.write
  fn write_all(self: mut Self, buf: Array<Byte>) -> Result<Unit, IoError> with Fs.write
  fn flush(self: mut Self) -> Result<Unit, IoError> with Fs.write
```

```
trait Seek
  doc: "a byte stream that supports repositioning"
  fn seek(self: mut Self, pos: SeekFrom) -> Result<Int, IoError> with Fs.read
```

### 12.2.2 Standard Streams

```
fn stdin() -> Stdin
  doc: "returns a handle to the standard input stream"
  effects: []

fn stdout() -> Stdout
  doc: "returns a handle to the standard output stream"
  effects: []

fn stderr() -> Stderr
  doc: "returns a handle to the standard error stream"
  effects: []
```

`Stdin` implements `Read`. `Stdout` and `Stderr` implement `Write`.

### 12.2.3 Buffered I/O

```
type BufReader<R: Read>
  doc: "wraps a reader with an internal buffer to reduce system calls"

fn BufReader.new(reader: R) -> BufReader<R>
  doc: "creates a buffered reader with default buffer size (8KB)"
  effects: []

fn BufReader.with_capacity(reader: R, capacity: Int) -> BufReader<R>
  doc: "creates a buffered reader with the specified buffer size"
  effects: []

impl BufReader<R: Read>
  fn read_line(self: mut Self) -> Result<Option<String>, IoError> with Fs.read
  fn lines(self: mut Self) -> Lines<Self> with Fs.read
```

```
type BufWriter<W: Write>
  doc: "wraps a writer with an internal buffer to reduce system calls"

fn BufWriter.new(writer: W) -> BufWriter<W>
  doc: "creates a buffered writer with default buffer size (8KB)"
  effects: []
```

### 12.2.4 Utility Functions

```
fn copy(reader: mut impl Read, writer: mut impl Write) -> Result<Int, IoError>
  doc: "copies all bytes from reader to writer, returns total bytes copied"
  effects: [Fs.read, Fs.write]
  panics: never
```

### 12.2.5 Error Type

```
struct IoError
  kind: IoErrorKind
  message: String
  os_error: Option<Int>

type IoErrorKind
  | NotFound
  | PermissionDenied
  | ConnectionRefused
  | ConnectionReset
  | BrokenPipe
  | AlreadyExists
  | WouldBlock
  | InvalidInput
  | InvalidData
  | TimedOut
  | UnexpectedEof
  | Interrupted
  | OutOfMemory
  | Other
```

---

## 12.3 `std/fs`: Filesystem

Provides filesystem operations. All functions carry the appropriate `Fs` effect.

### 12.3.1 File Type

```
type File
  doc: "an owned handle to an open file"

fn File.open(path: String) -> Result<File, IoError>
  doc: "opens a file for reading"
  effects: [Fs.read]
  panics: never

fn File.create(path: String) -> Result<File, IoError>
  doc: "creates a file for writing, truncating if it exists"
  effects: [Fs.write]
  panics: never

fn File.open_with(path: String, options: OpenOptions) -> Result<File, IoError>
  doc: "opens a file with the specified options"
  effects: [Fs.read | Fs.write]
  panics: never

impl Read for File
impl Write for File
impl Seek for File
impl Drop for File
```

### 12.3.2 Filesystem Operations

```
fn read_to_string(path: String) -> Result<String, IoError>
  doc: "reads the entire contents of a file as a UTF-8 string"
  effects: [Fs.read]
  panics: never

fn read_bytes(path: String) -> Result<Vec<Byte>, IoError>
  doc: "reads the entire contents of a file as bytes"
  effects: [Fs.read]
  panics: never

fn write(path: String, contents: Array<Byte>) -> Result<Unit, IoError>
  doc: "writes bytes to a file, creating it if it doesn't exist, truncating if it does"
  effects: [Fs.write]
  panics: never

fn write_string(path: String, contents: String) -> Result<Unit, IoError>
  doc: "writes a string to a file"
  effects: [Fs.write]
  panics: never

fn append(path: String, contents: Array<Byte>) -> Result<Unit, IoError>
  doc: "appends bytes to a file, creating it if it doesn't exist"
  effects: [Fs.write]
  panics: never

fn copy(from: String, to: String) -> Result<Unit, IoError>
  doc: "copies a file from one path to another"
  effects: [Fs.read, Fs.write]
  panics: never

fn rename(from: String, to: String) -> Result<Unit, IoError>
  doc: "renames a file or directory"
  effects: [Fs.write]
  panics: never

fn remove(path: String) -> Result<Unit, IoError>
  doc: "removes a file"
  effects: [Fs.write]
  panics: never

fn create_dir(path: String) -> Result<Unit, IoError>
  doc: "creates a directory"
  effects: [Fs.write]
  panics: never

fn create_dir_all(path: String) -> Result<Unit, IoError>
  doc: "creates a directory and all parent directories"
  effects: [Fs.write]
  panics: never

fn remove_dir(path: String) -> Result<Unit, IoError>
  doc: "removes an empty directory"
  effects: [Fs.write]
  panics: never

fn remove_dir_all(path: String) -> Result<Unit, IoError>
  doc: "removes a directory and all its contents"
  effects: [Fs.write]
  panics: never

fn exists(path: String) -> Bool
  doc: "returns true if the path exists"
  effects: [Fs.read]
  panics: never

fn metadata(path: String) -> Result<Metadata, IoError>
  doc: "returns metadata for a file or directory"
  effects: [Fs.read]
  panics: never
```

### 12.3.3 Directory Walking

```
fn walk(path: String) -> Result<Walker, IoError>
  doc: "returns an iterator over directory entries, recursively"
  effects: [Fs.read]
  panics: never

fn read_dir(path: String) -> Result<DirEntries, IoError>
  doc: "returns an iterator over entries in a directory (non-recursive)"
  effects: [Fs.read]
  panics: never

struct DirEntry
  path: String
  name: String
  kind: FileKind
  metadata: Metadata

type FileKind
  | File
  | Directory
  | Symlink
  | Other

struct Metadata
  size: UInt64
  modified: Instant
  created: Option<Instant>
  accessed: Option<Instant>
  permissions: Permissions
  kind: FileKind
```

---

## 12.4 `std/net`: Networking

Provides TCP and UDP networking. All operations carry the `Net` effect.

### 12.4.1 TCP

```
type TcpStream
  doc: "a TCP connection between a local and remote socket"

fn TcpStream.connect(addr: String) -> Result<TcpStream, IoError>
  doc: "opens a TCP connection to the given address"
  effects: [Net.connect]
  panics: never

impl Read for TcpStream
impl Write for TcpStream
impl Drop for TcpStream

type TcpListener
  doc: "a TCP socket server that listens for connections"

fn TcpListener.bind(addr: String) -> Result<TcpListener, IoError>
  doc: "binds a TCP listener to the given address"
  effects: [Net.listen]
  panics: never

fn TcpListener.accept(self: Self) -> Result<(TcpStream, SocketAddr), IoError>
  doc: "accepts a new incoming connection"
  effects: [Net.accept]
  panics: never

fn TcpListener.incoming(self: Self) -> Incoming
  doc: "returns an iterator over incoming connections"
  effects: [Net.accept]
  panics: never
```

### 12.4.2 UDP

```
type UdpSocket
  doc: "a UDP socket"

fn UdpSocket.bind(addr: String) -> Result<UdpSocket, IoError>
  doc: "binds a UDP socket to the given address"
  effects: [Net.listen]
  panics: never

fn UdpSocket.send_to(self: Self, buf: Array<Byte>, addr: String) -> Result<Int, IoError>
  doc: "sends bytes to the given address"
  effects: [Net.send]
  panics: never

fn UdpSocket.recv_from(self: Self, buf: mut Array<Byte>) -> Result<(Int, SocketAddr), IoError>
  doc: "receives bytes and returns the sender's address"
  effects: [Net.recv]
  panics: never
```

### 12.4.3 Address Types

```
type SocketAddr
  | V4(Ipv4Addr, UInt16)
  | V6(Ipv6Addr, UInt16)

struct Ipv4Addr
  octets: Array<Byte, 4>

struct Ipv6Addr
  octets: Array<Byte, 16>
```

---

## 12.5 `std/http`: HTTP

Provides HTTP client and server types. Built on `std/net` and `std/async`.

### 12.5.1 Client

```
type HttpClient
  doc: "an HTTP client for making requests"

fn HttpClient.new() -> HttpClient
  doc: "creates a new HTTP client with default configuration"
  effects: []

fn HttpClient.get(self: Self, url: String) -> Result<Response, HttpError>
  doc: "sends an HTTP GET request"
  effects: [Net.connect, Net.send, Net.recv]
  panics: never

fn HttpClient.post(self: Self, url: String, body: Array<Byte>) -> Result<Response, HttpError>
  doc: "sends an HTTP POST request with the given body"
  effects: [Net.connect, Net.send, Net.recv]
  panics: never

fn HttpClient.request(self: Self, req: Request) -> Result<Response, HttpError>
  doc: "sends an arbitrary HTTP request"
  effects: [Net.connect, Net.send, Net.recv]
  panics: never
```

### 12.5.2 Request and Response

```
struct Request
  method: Method
  url: String
  headers: Headers
  body: Option<Vec<Byte>>

struct Response
  status: StatusCode
  headers: Headers
  body: Vec<Byte>

impl Response
  fn text(self: Self) -> Result<String, HttpError>
  fn json<T: Deserialize>(self: Self) -> Result<T, HttpError>
  fn status(self: Self) -> StatusCode
  fn is_success(self: Self) -> Bool

type Method
  | Get
  | Post
  | Put
  | Delete
  | Patch
  | Head
  | Options

struct StatusCode(UInt16)

impl StatusCode
  fn is_success(self: Self) -> Bool      // 200-299
  fn is_redirect(self: Self) -> Bool     // 300-399
  fn is_client_error(self: Self) -> Bool // 400-499
  fn is_server_error(self: Self) -> Bool // 500-599

struct Headers
  // case-insensitive header map
  fn get(self: Self, name: String) -> Option<String>
  fn set(self: mut Self, name: String, value: String)
  fn append(self: mut Self, name: String, value: String)
  fn iter(self: Self) -> Iterator<Item = (String, String)>
```

### 12.5.3 Server

```
type HttpServer
  doc: "an HTTP server that listens for and handles requests"

fn HttpServer.bind(addr: String) -> Result<HttpServer, HttpError>
  doc: "binds the server to the given address"
  effects: [Net.listen]
  panics: never

fn HttpServer.serve(self: mut Self, handler: fn(Request) -> Response) -> Result<Unit, HttpError>
  doc: "starts serving requests, calling the handler for each"
  effects: [Net.accept, Net.send, Net.recv, Async]
  panics: never
```

---

## 12.6 `std/json`: JSON

Provides JSON serialization and deserialization.

### 12.6.1 Core Functions

```
fn parse<T: Deserialize>(s: String) -> Result<T, JsonError>
  doc: "parses a JSON string into a value of type T"
  effects: []
  panics: never

fn parse_value(s: String) -> Result<JsonValue, JsonError>
  doc: "parses a JSON string into a dynamic JSON value"
  effects: []
  panics: never

fn to_string<T: Serialize>(value: T) -> Result<String, JsonError>
  doc: "serializes a value to a compact JSON string"
  effects: []
  panics: never

fn to_string_pretty<T: Serialize>(value: T) -> Result<String, JsonError>
  doc: "serializes a value to a pretty-printed JSON string"
  effects: []
  panics: never

fn from_value<T: Deserialize>(value: JsonValue) -> Result<T, JsonError>
  doc: "converts a JsonValue into a typed value"
  effects: []
  panics: never

fn to_value<T: Serialize>(value: T) -> Result<JsonValue, JsonError>
  doc: "converts a typed value into a JsonValue"
  effects: []
  panics: never
```

### 12.6.2 Dynamic JSON Type

```
type JsonValue
  | Null
  | Bool(Bool)
  | Number(Float)
  | String(String)
  | Array(Vec<JsonValue>)
  | Object(Map<String, JsonValue>)

impl JsonValue
  fn is_null(self: Self) -> Bool
  fn as_bool(self: Self) -> Option<Bool>
  fn as_number(self: Self) -> Option<Float>
  fn as_str(self: Self) -> Option<String>
  fn as_array(self: Self) -> Option<Vec<JsonValue>>
  fn as_object(self: Self) -> Option<Map<String, JsonValue>>
  fn get(self: Self, key: String) -> Option<JsonValue>
  fn index(self: Self, index: Int) -> Option<JsonValue>
```

### 12.6.3 Serialization Traits

```
trait Serialize
  doc: "a type that can be serialized to JSON"
  fn serialize(self: Self, serializer: mut Serializer) -> Result<Unit, JsonError>

trait Deserialize
  doc: "a type that can be deserialized from JSON"
  fn deserialize(deserializer: mut Deserializer) -> Result<Self, JsonError>
```

The `@derive(Serialize, Deserialize)` attribute auto-generates implementations for structs and enums:

```
@derive(Serialize, Deserialize)
struct User
  name: String
  email: String
  age: Int
```

### 12.6.4 Error Type

```
struct JsonError
  message: String
  line: Int
  col: Int
  kind: JsonErrorKind

type JsonErrorKind
  | SyntaxError
  | TypeError
  | MissingField
  | UnknownField
  | Overflow
  | Eof
```

---

## 12.7 `std/text`: Text Processing

String manipulation, regex, and formatting utilities.

### 12.7.1 String Extensions

Monel's `String` type is UTF-8 encoded and provides rich manipulation methods:

```
impl String
  // Construction
  fn new() -> String
  fn from_bytes(bytes: Vec<Byte>) -> Result<String, Utf8Error>
  fn from_bytes_unchecked(bytes: Vec<Byte>) -> String with unsafe
  fn repeat(s: String, n: Int) -> String

  // Queries
  fn len(self: Self) -> Int                    // byte length
  fn char_count(self: Self) -> Int             // Unicode scalar count
  fn is_empty(self: Self) -> Bool
  fn contains(self: Self, pattern: String) -> Bool
  fn starts_with(self: Self, prefix: String) -> Bool
  fn ends_with(self: Self, suffix: String) -> Bool
  fn find(self: Self, pattern: String) -> Option<Int>
  fn rfind(self: Self, pattern: String) -> Option<Int>

  // Transformation
  fn to_uppercase(self: Self) -> String
  fn to_lowercase(self: Self) -> String
  fn trim(self: Self) -> String
  fn trim_start(self: Self) -> String
  fn trim_end(self: Self) -> String
  fn replace(self: Self, from: String, to: String) -> String
  fn replacen(self: Self, from: String, to: String, count: Int) -> String

  // Splitting
  fn split(self: Self, separator: String) -> Vec<String>
  fn splitn(self: Self, separator: String, n: Int) -> Vec<String>
  fn lines(self: Self) -> Vec<String>
  fn chars(self: Self) -> Iterator<Item = Char>
  fn bytes(self: Self) -> Iterator<Item = Byte>

  // Joining
  fn join(parts: Array<String>, separator: String) -> String

  // Slicing
  fn slice(self: Self, start: Int, end: Int) -> String
  fn as_bytes(self: Self) -> Array<Byte>
```

### 12.7.2 Regex

```
type Regex
  doc: "a compiled regular expression"

fn Regex.new(pattern: String) -> Result<Regex, RegexError>
  doc: "compiles a regular expression pattern"
  effects: []
  panics: never

impl Regex
  fn is_match(self: Self, text: String) -> Bool
  fn find(self: Self, text: String) -> Option<Match>
  fn find_all(self: Self, text: String) -> Vec<Match>
  fn captures(self: Self, text: String) -> Option<Captures>
  fn replace(self: Self, text: String, replacement: String) -> String
  fn replace_all(self: Self, text: String, replacement: String) -> String
  fn split(self: Self, text: String) -> Vec<String>

struct Match
  start: Int
  end: Int
  text: String

struct Captures
  fn get(self: Self, index: Int) -> Option<Match>
  fn name(self: Self, name: String) -> Option<Match>
  fn len(self: Self) -> Int
```

### 12.7.3 String Formatting

```
fn format(template: String, args: ...) -> String
  doc: "formats a string using positional or named arguments"
  effects: []
  panics: "if format string is invalid or argument count mismatches"
```

Format syntax: `{}` for positional, `{name}` for named, `{:spec}` for format specifiers:

```
format("hello, {}!", name)
format("{name} is {age} years old", name=name, age=age)
format("{:>10}", value)    // right-align, width 10
format("{:.2}", 3.14159)   // 2 decimal places
format("{:#x}", 255)       // hex: 0xff
format("{:#b}", 42)        // binary: 0b101010
```

---

## 12.8 `std/math`: Mathematics

### 12.8.1 Constants

```
const PI: Float = 3.141592653589793
const E: Float = 2.718281828459045
const TAU: Float = 6.283185307179586
const INFINITY: Float = Float.INFINITY
const NEG_INFINITY: Float = Float.NEG_INFINITY
const NAN: Float = Float.NAN
```

### 12.8.2 Functions

```
fn abs(x: Float) -> Float
fn ceil(x: Float) -> Float
fn floor(x: Float) -> Float
fn round(x: Float) -> Float
fn trunc(x: Float) -> Float

fn sqrt(x: Float) -> Float
fn cbrt(x: Float) -> Float
fn pow(base: Float, exp: Float) -> Float
fn exp(x: Float) -> Float
fn ln(x: Float) -> Float
fn log(x: Float, base: Float) -> Float
fn log2(x: Float) -> Float
fn log10(x: Float) -> Float

fn sin(x: Float) -> Float
fn cos(x: Float) -> Float
fn tan(x: Float) -> Float
fn asin(x: Float) -> Float
fn acos(x: Float) -> Float
fn atan(x: Float) -> Float
fn atan2(y: Float, x: Float) -> Float

fn min(a: Float, b: Float) -> Float
fn max(a: Float, b: Float) -> Float
fn clamp(x: Float, min: Float, max: Float) -> Float
```

### 12.8.3 Integer Math

```
impl Int
  fn abs(self) -> Int
  fn pow(self, exp: UInt32) -> Int
  fn div_euclid(self, rhs: Int) -> Int
  fn rem_euclid(self, rhs: Int) -> Int
  fn checked_add(self, rhs: Int) -> Option<Int>
  fn checked_sub(self, rhs: Int) -> Option<Int>
  fn checked_mul(self, rhs: Int) -> Option<Int>
  fn checked_div(self, rhs: Int) -> Option<Int>
  fn saturating_add(self, rhs: Int) -> Int
  fn saturating_sub(self, rhs: Int) -> Int
  fn saturating_mul(self, rhs: Int) -> Int
  fn wrapping_add(self, rhs: Int) -> Int
  fn wrapping_sub(self, rhs: Int) -> Int
  fn wrapping_mul(self, rhs: Int) -> Int
  fn count_ones(self) -> UInt32
  fn count_zeros(self) -> UInt32
  fn leading_zeros(self) -> UInt32
  fn trailing_zeros(self) -> UInt32
```

---

## 12.9 `std/time`: Time

### 12.9.1 Core Types

```
type Duration
  doc: "a span of time with nanosecond precision"

impl Duration
  fn from_secs(secs: UInt64) -> Duration
  fn from_millis(millis: UInt64) -> Duration
  fn from_micros(micros: UInt64) -> Duration
  fn from_nanos(nanos: UInt64) -> Duration
  fn as_secs(self: Self) -> UInt64
  fn as_millis(self: Self) -> UInt64
  fn as_micros(self: Self) -> UInt64
  fn as_nanos(self: Self) -> UInt128
  fn is_zero(self: Self) -> Bool
  fn checked_add(self: Self, other: Duration) -> Option<Duration>
  fn checked_sub(self: Self, other: Duration) -> Option<Duration>
  fn checked_mul(self: Self, factor: UInt32) -> Option<Duration>
```

Duration literals are supported in Monel syntax:

```
let timeout = 30s
let interval = 100ms
let precision = 50us
let tick = 1ns
let long_wait = 5m
let very_long = 2h
```

```
type Instant
  doc: "a monotonic timestamp for measuring elapsed time"

fn Instant.now() -> Instant
  doc: "returns the current monotonic time"
  effects: [Time.now]

impl Instant
  fn elapsed(self: Self) -> Duration
  fn duration_since(self: Self, earlier: Instant) -> Duration
  fn checked_add(self: Self, duration: Duration) -> Option<Instant>
  fn checked_sub(self: Self, duration: Duration) -> Option<Instant>
```

```
type Clock
  doc: "a wall-clock timestamp (not monotonic, subject to NTP adjustments)"

fn Clock.now() -> Clock
  doc: "returns the current wall-clock time"
  effects: [Time.now]

impl Clock
  fn year(self: Self) -> Int
  fn month(self: Self) -> Int
  fn day(self: Self) -> Int
  fn hour(self: Self) -> Int
  fn minute(self: Self) -> Int
  fn second(self: Self) -> Int
  fn timestamp(self: Self) -> Int64         // Unix timestamp (seconds)
  fn timestamp_millis(self: Self) -> Int64  // Unix timestamp (milliseconds)
  fn format(self: Self, fmt: String) -> String
  fn parse(s: String, fmt: String) -> Result<Clock, TimeError>
```

### 12.9.2 Timers

```
fn sleep(duration: Duration) -> Unit
  doc: "suspends the current task for the given duration"
  effects: [Time.sleep, Async]

fn interval(period: Duration) -> Interval
  doc: "creates a repeating interval timer"
  effects: [Time.now]

fn timeout<T>(duration: Duration, future: impl Future<T>) -> Result<T, TimeoutError>
  doc: "wraps a future with a timeout, returning TimeoutError if it exceeds the duration"
  effects: [Time.now, Async]
```

---

## 12.10 `std/collections`: Additional Collections

Beyond the core `Vec`, `Map`, and `Set`, this module provides specialized collection types.

### 12.10.1 Ordered Collections

```
type BTreeMap<K: Ord, V>
  doc: "a sorted map implemented as a B-tree"

impl BTreeMap<K: Ord, V>
  fn new() -> BTreeMap<K, V>
  fn insert(self: mut Self, key: K, value: V) -> Option<V>
  fn get(self: Self, key: K) -> Option<V>
  fn remove(self: mut Self, key: K) -> Option<V>
  fn contains_key(self: Self, key: K) -> Bool
  fn len(self: Self) -> Int
  fn is_empty(self: Self) -> Bool
  fn range(self: Self, range: impl RangeBounds<K>) -> Iterator<Item = (K, V)>
  fn iter(self: Self) -> Iterator<Item = (K, V)>    // sorted order
  fn keys(self: Self) -> Iterator<Item = K>
  fn values(self: Self) -> Iterator<Item = V>
```

```
type BTreeSet<T: Ord>
  doc: "a sorted set implemented as a B-tree"

impl BTreeSet<T: Ord>
  fn new() -> BTreeSet<T>
  fn insert(self: mut Self, value: T) -> Bool
  fn remove(self: mut Self, value: T) -> Bool
  fn contains(self: Self, value: T) -> Bool
  fn len(self: Self) -> Int
  fn range(self: Self, range: impl RangeBounds<T>) -> Iterator<Item = T>
  fn iter(self: Self) -> Iterator<Item = T>          // sorted order
  fn union(self: Self, other: BTreeSet<T>) -> BTreeSet<T>
  fn intersection(self: Self, other: BTreeSet<T>) -> BTreeSet<T>
  fn difference(self: Self, other: BTreeSet<T>) -> BTreeSet<T>
```

### 12.10.2 Double-Ended Queue

```
type Deque<T>
  doc: "a double-ended queue implemented as a ring buffer"

impl Deque<T>
  fn new() -> Deque<T>
  fn push_front(self: mut Self, value: T)
  fn push_back(self: mut Self, value: T)
  fn pop_front(self: mut Self) -> Option<T>
  fn pop_back(self: mut Self) -> Option<T>
  fn front(self: Self) -> Option<T>
  fn back(self: Self) -> Option<T>
  fn len(self: Self) -> Int
  fn is_empty(self: Self) -> Bool
  fn iter(self: Self) -> Iterator<Item = T>
```

### 12.10.3 Linked List

```
type LinkedList<T>
  doc: "a doubly-linked list"

impl LinkedList<T>
  fn new() -> LinkedList<T>
  fn push_front(self: mut Self, value: T)
  fn push_back(self: mut Self, value: T)
  fn pop_front(self: mut Self) -> Option<T>
  fn pop_back(self: mut Self) -> Option<T>
  fn len(self: Self) -> Int
  fn is_empty(self: Self) -> Bool
  fn iter(self: Self) -> Iterator<Item = T>
```

### 12.10.4 Priority Queue

```
type PriorityQueue<T: Ord>
  doc: "a max-heap priority queue"

impl PriorityQueue<T: Ord>
  fn new() -> PriorityQueue<T>
  fn push(self: mut Self, value: T)
  fn pop(self: mut Self) -> Option<T>       // removes and returns max
  fn peek(self: Self) -> Option<T>          // returns max without removing
  fn len(self: Self) -> Int
  fn is_empty(self: Self) -> Bool
```

### 12.10.5 Hash Map and Hash Set (Core)

The core `Map` and `Set` types use hash-based implementation:

```
impl Map<K: Hash + Eq, V>
  fn new() -> Map<K, V>
  fn with_capacity(capacity: Int) -> Map<K, V>
  fn insert(self: mut Self, key: K, value: V) -> Option<V>
  fn get(self: Self, key: K) -> Option<V>
  fn get_mut(self: mut Self, key: K) -> Option<mut V>
  fn remove(self: mut Self, key: K) -> Option<V>
  fn contains_key(self: Self, key: K) -> Bool
  fn len(self: Self) -> Int
  fn is_empty(self: Self) -> Bool
  fn keys(self: Self) -> Iterator<Item = K>
  fn values(self: Self) -> Iterator<Item = V>
  fn iter(self: Self) -> Iterator<Item = (K, V)>
  fn entry(self: mut Self, key: K) -> Entry<K, V>

type Entry<K, V>
  | Occupied(OccupiedEntry<K, V>)
  | Vacant(VacantEntry<K, V>)

impl Entry<K, V>
  fn or_insert(self, default: V) -> mut V
  fn or_insert_with(self, f: fn() -> V) -> mut V
```

```
impl Set<T: Hash + Eq>
  fn new() -> Set<T>
  fn insert(self: mut Self, value: T) -> Bool
  fn remove(self: mut Self, value: T) -> Bool
  fn contains(self: Self, value: T) -> Bool
  fn len(self: Self) -> Int
  fn is_empty(self: Self) -> Bool
  fn iter(self: Self) -> Iterator<Item = T>
  fn union(self: Self, other: Set<T>) -> Set<T>
  fn intersection(self: Self, other: Set<T>) -> Set<T>
  fn difference(self: Self, other: Set<T>) -> Set<T>
  fn symmetric_difference(self: Self, other: Set<T>) -> Set<T>
  fn is_subset(self: Self, other: Set<T>) -> Bool
  fn is_superset(self: Self, other: Set<T>) -> Bool
```

---

## 12.11 `std/sync`: Synchronization

Synchronization primitives for concurrent programming.

### 12.11.1 Mutex

```
type Mutex<T>
  doc: "a mutual exclusion lock protecting shared data"

fn Mutex.new(value: T) -> Mutex<T>
  doc: "creates a new mutex wrapping the given value"
  effects: []

fn Mutex.lock(self: Self) -> MutexGuard<T>
  doc: "acquires the lock, blocking until it is available"
  effects: [Sync.lock]

fn Mutex.try_lock(self: Self) -> Option<MutexGuard<T>>
  doc: "attempts to acquire the lock without blocking"
  effects: [Sync.lock]

struct MutexGuard<T>
  // Dereferences to mut T. Releases the lock on drop.
```

### 12.11.2 Read-Write Lock

```
type RwLock<T>
  doc: "a reader-writer lock allowing multiple readers or a single writer"

fn RwLock.new(value: T) -> RwLock<T>
  doc: "creates a new read-write lock wrapping the given value"
  effects: []

fn RwLock.read(self: Self) -> RwLockReadGuard<T>
  doc: "acquires a read lock, blocking until no writer holds the lock"
  effects: [Sync.lock]

fn RwLock.write(self: Self) -> RwLockWriteGuard<T>
  doc: "acquires a write lock, blocking until no reader or writer holds the lock"
  effects: [Sync.lock]
```

### 12.11.3 Atomics

```
type Atomic<T>
  doc: "an atomically-accessible value"

impl Atomic<T>
  fn new(value: T) -> Atomic<T>
  fn load(self: Self, order: Ordering) -> T
  fn store(self: Self, value: T, order: Ordering)
  fn swap(self: Self, value: T, order: Ordering) -> T
  fn compare_exchange(self: Self, expected: T, new: T, success: Ordering, failure: Ordering) -> Result<T, T>
  fn fetch_add(self: Self, value: T, order: Ordering) -> T    // for integer T
  fn fetch_sub(self: Self, value: T, order: Ordering) -> T    // for integer T
  fn fetch_and(self: Self, value: T, order: Ordering) -> T    // for integer T
  fn fetch_or(self: Self, value: T, order: Ordering) -> T     // for integer T

type Ordering
  | Relaxed
  | Acquire
  | Release
  | AcqRel
  | SeqCst
```

### 12.11.4 Channels

```
fn channel<T>() -> (Sender<T>, Receiver<T>)
  doc: "creates an unbounded MPSC channel"
  effects: []

fn bounded_channel<T>(capacity: Int) -> (Sender<T>, Receiver<T>)
  doc: "creates a bounded MPSC channel with the given capacity"
  effects: []

type Sender<T>
  doc: "the sending half of a channel"

impl Sender<T>
  fn send(self: Self, value: T) -> Result<Unit, SendError<T>>
  fn try_send(self: Self, value: T) -> Result<Unit, TrySendError<T>>  // bounded only
  fn is_closed(self: Self) -> Bool

impl Clone for Sender<T>  // multiple senders allowed

type Receiver<T>
  doc: "the receiving half of a channel"

impl Receiver<T>
  fn recv(self: Self) -> Result<T, RecvError>
  fn try_recv(self: Self) -> Result<T, TryRecvError>
  fn iter(self: Self) -> Iterator<Item = T>
```

### 12.11.5 Once

```
type Once
  doc: "a synchronization primitive for one-time initialization"

impl Once
  fn new() -> Once
  fn call_once(self: Self, f: fn() -> Unit)
  fn is_completed(self: Self) -> Bool

type OnceCell<T>
  doc: "a cell that can be written to exactly once"

impl OnceCell<T>
  fn new() -> OnceCell<T>
  fn get(self: Self) -> Option<T>
  fn set(self: Self, value: T) -> Result<Unit, T>
  fn get_or_init(self: Self, f: fn() -> T) -> T
```

---

## 12.12 `std/async`: Async Runtime

Provides the async runtime, task spawning, and event loop integration.

### 12.12.1 Core Types

```
type Runtime
  doc: "the async runtime that drives task execution"

fn Runtime.new() -> Result<Runtime, IoError>
  doc: "creates a new async runtime with default configuration"
  effects: [unsafe]
  panics: never

fn Runtime.block_on<T>(self: mut Self, future: impl Future<T>) -> T
  doc: "runs a future to completion on the runtime, blocking the current thread"
  effects: [Async]
```

### 12.12.2 Task Spawning

```
fn spawn<T>(future: impl Future<T>) -> JoinHandle<T>
  doc: "spawns a new async task on the runtime"
  effects: [Async]

fn spawn_blocking<T>(f: fn() -> T) -> JoinHandle<T>
  doc: "runs a blocking function on a dedicated thread pool"
  effects: [Async]

type JoinHandle<T>
  doc: "a handle to a spawned task, can be awaited for its result"

impl JoinHandle<T>
  fn await(self) -> Result<T, JoinError>
  fn abort(self: Self)
  fn is_finished(self: Self) -> Bool
```

### 12.12.3 Select

```
macro select
  doc: "waits on multiple futures simultaneously, completing when the first resolves"
```

```
select
  result = http_response.await =>
    handle_response(result)
  _ = timeout(5s).await =>
    handle_timeout()
  msg = channel.recv().await =>
    handle_message(msg)
```

### 12.12.4 Event Loop

```
type EventLoop
  doc: "a low-level event loop for I/O multiplexing"

fn EventLoop.new() -> Result<EventLoop, IoError>
  doc: "creates a new event loop backed by epoll (Linux) or kqueue (macOS)"
  effects: [unsafe]
  panics: never

impl EventLoop
  fn run(self: mut Self, handler: fn(Event) -> LoopControl) -> Result<Unit, IoError> with unsafe, Async
  fn register_fd(self: mut Self, fd: Fd, interest: IoReadiness) -> Result<Unit, IoError> with unsafe
  fn deregister_fd(self: mut Self, fd: Fd) -> Result<Unit, IoError> with unsafe
  fn register_timer(self: mut Self, duration: Duration, repeat: Bool) -> TimerId
  fn cancel_timer(self: mut Self, id: TimerId)
  fn register_signal(self: mut Self, sig: SignalKind) -> Result<Unit, IoError> with unsafe, Signal
  fn wake(self: Self)  // wake the event loop from another thread

type IoReadiness
  | Read
  | Write
  | ReadWrite

type Event
  | IoReady(Fd, IoReadiness)
  | Timer(TimerId)
  | Signal(SignalKind)
  | Wake
  | Custom(Box<dyn Any>)

type LoopControl
  | Continue
  | Break
```

---

## 12.13 `std/crypto`: Cryptography

Provides hashing, encryption, and signing primitives.

### 12.13.1 Hashing

```
fn hash_sha256(data: Array<Byte>) -> Array<Byte, 32>
  doc: "computes the SHA-256 hash of the given data"
  effects: [Crypto]
  panics: never

fn hash_sha512(data: Array<Byte>) -> Array<Byte, 64>
  doc: "computes the SHA-512 hash of the given data"
  effects: [Crypto]
  panics: never

fn hash_blake3(data: Array<Byte>) -> Array<Byte, 32>
  doc: "computes the BLAKE3 hash of the given data"
  effects: [Crypto]
  panics: never

type Hasher
  doc: "an incremental hash computation"

impl Hasher
  fn new(algorithm: HashAlgorithm) -> Hasher
  fn update(self: mut Self, data: Array<Byte>)
  fn finalize(self) -> Vec<Byte>

type HashAlgorithm
  | Sha256
  | Sha384
  | Sha512
  | Blake3
```

### 12.13.2 Encryption

```
fn encrypt_aes_gcm(key: Array<Byte>, nonce: Array<Byte>, plaintext: Array<Byte>, aad: Array<Byte>) -> Result<Vec<Byte>, CryptoError>
  doc: "encrypts data using AES-256-GCM"
  effects: [Crypto]
  panics: never

fn decrypt_aes_gcm(key: Array<Byte>, nonce: Array<Byte>, ciphertext: Array<Byte>, aad: Array<Byte>) -> Result<Vec<Byte>, CryptoError>
  doc: "decrypts data using AES-256-GCM"
  effects: [Crypto]
  panics: never
```

### 12.13.3 Signing

```
fn sign_ed25519(private_key: Array<Byte>, message: Array<Byte>) -> Result<Array<Byte, 64>, CryptoError>
  doc: "signs a message using Ed25519"
  effects: [Crypto]
  panics: never

fn verify_ed25519(public_key: Array<Byte>, message: Array<Byte>, signature: Array<Byte>) -> Result<Bool, CryptoError>
  doc: "verifies an Ed25519 signature"
  effects: [Crypto]
  panics: never
```

### 12.13.4 Random

```
fn random_bytes(len: Int) -> Vec<Byte>
  doc: "generates cryptographically secure random bytes"
  effects: [Crypto, Random]
  panics: never

fn random_int(min: Int, max: Int) -> Int
  doc: "generates a random integer in the range [min, max)"
  effects: [Random]
  panics: "if min >= max"
```

---

## 12.14 `std/env`: Environment

```
fn var(name: String) -> Result<String, EnvError>
  doc: "returns the value of an environment variable"
  effects: [Env.read]
  panics: never

fn var_or(name: String, default: String) -> String
  doc: "returns the value of an environment variable, or the default if not set"
  effects: [Env.read]
  panics: never

fn set_var(name: String, value: String)
  doc: "sets an environment variable"
  effects: [Env.write]

fn remove_var(name: String)
  doc: "removes an environment variable"
  effects: [Env.write]

fn vars() -> Iterator<Item = (String, String)>
  doc: "returns an iterator over all environment variables"
  effects: [Env.read]

fn args() -> Vec<String>
  doc: "returns the command-line arguments"
  effects: [Env.read]

fn current_dir() -> Result<String, IoError>
  doc: "returns the current working directory"
  effects: [Fs.read]

fn home_dir() -> Option<String>
  doc: "returns the user's home directory"
  effects: [Env.read]

fn temp_dir() -> String
  doc: "returns the system's temporary directory"
  effects: [Env.read]
```

---

## 12.15 `std/process`: Process Management

```
type Command
  doc: "a builder for spawning child processes"

fn Command.new(program: String) -> Command
  doc: "creates a new command for the given program"
  effects: []

impl Command
  fn arg(self: mut Self, arg: String) -> mut Self
  fn args(self: mut Self, args: Array<String>) -> mut Self
  fn env(self: mut Self, key: String, value: String) -> mut Self
  fn current_dir(self: mut Self, dir: String) -> mut Self
  fn stdin(self: mut Self, cfg: Stdio) -> mut Self
  fn stdout(self: mut Self, cfg: Stdio) -> mut Self
  fn stderr(self: mut Self, cfg: Stdio) -> mut Self

fn Command.spawn(self: mut Self) -> Result<Child, IoError>
  doc: "spawns the process without waiting for it to complete"
  effects: [Process.spawn]
  panics: never

fn Command.output(self: mut Self) -> Result<Output, IoError>
  doc: "spawns the process and waits for it, collecting all output"
  effects: [Process.spawn, Process.wait]
  panics: never

fn Command.status(self: mut Self) -> Result<ExitStatus, IoError>
  doc: "spawns the process and waits for it, returning its exit status"
  effects: [Process.spawn, Process.wait]
  panics: never

struct Child
  stdin: Option<ChildStdin>
  stdout: Option<ChildStdout>
  stderr: Option<ChildStderr>

impl Child
  fn wait(self: mut Self) -> Result<ExitStatus, IoError> with Process.wait
  fn kill(self: mut Self) -> Result<Unit, IoError> with Process.signal
  fn id(self: Self) -> UInt32

struct Output
  status: ExitStatus
  stdout: Vec<Byte>
  stderr: Vec<Byte>

struct ExitStatus
  fn success(self: Self) -> Bool
  fn code(self: Self) -> Option<Int>

type Stdio
  | Inherit
  | Piped
  | Null
```

---

## 12.16 `std/terminal`: Terminal I/O

Terminal-specific I/O for building terminal applications.

### 12.16.1 Terminal Control

```
fn size() -> Result<TermSize, IoError>
  doc: "returns the current terminal size in rows and columns"
  effects: [Fs.read, unsafe]
  panics: never

fn enable_raw_mode() -> Result<RawModeGuard, IoError>
  doc: "enables raw mode for stdin (no echo, no line buffering)"
  effects: [Fs.write, unsafe]
  panics: never

struct RawModeGuard
  // restores original terminal mode on drop

struct TermSize
  rows: UInt16
  cols: UInt16
  pixel_width: UInt16
  pixel_height: UInt16
```

### 12.16.2 ANSI Escape Codes

```
module terminal/ansi
  doc: "ANSI escape code generation for terminal formatting"

fn cursor_to(row: Int, col: Int) -> String
fn cursor_up(n: Int) -> String
fn cursor_down(n: Int) -> String
fn cursor_left(n: Int) -> String
fn cursor_right(n: Int) -> String
fn cursor_save() -> String
fn cursor_restore() -> String
fn cursor_hide() -> String
fn cursor_show() -> String

fn clear_screen() -> String
fn clear_line() -> String
fn clear_to_end() -> String
fn clear_to_start() -> String

fn bold(text: String) -> String
fn dim(text: String) -> String
fn italic(text: String) -> String
fn underline(text: String) -> String
fn strikethrough(text: String) -> String
fn fg(text: String, color: Color) -> String
fn bg(text: String, color: Color) -> String

fn enter_alternate_screen() -> String
fn exit_alternate_screen() -> String

fn enable_mouse_capture() -> String
fn disable_mouse_capture() -> String
fn enable_bracketed_paste() -> String
fn disable_bracketed_paste() -> String
```

### 12.16.3 Input Parsing

```
fn read_event(timeout: Option<Duration>) -> Result<Option<InputEvent>, IoError>
  doc: "reads a single input event from stdin, with optional timeout"
  effects: [Fs.read, unsafe]
  panics: never

type InputEvent
  | Key(KeyEvent)
  | Mouse(MouseEvent)
  | Paste(String)
  | Resize(UInt16, UInt16)
  | FocusGained
  | FocusLost

struct KeyEvent
  code: KeyCode
  modifiers: Modifiers

type KeyCode
  | Char(Char)
  | Enter
  | Backspace
  | Delete
  | Tab
  | BackTab
  | Escape
  | Up
  | Down
  | Left
  | Right
  | Home
  | End
  | PageUp
  | PageDown
  | Insert
  | F(UInt8)
  | Null

struct Modifiers
  shift: Bool
  ctrl: Bool
  alt: Bool
  super_key: Bool

struct MouseEvent
  kind: MouseEventKind
  col: UInt16
  row: UInt16
  modifiers: Modifiers

type MouseEventKind
  | Down(MouseButton)
  | Up(MouseButton)
  | Drag(MouseButton)
  | Moved
  | ScrollUp
  | ScrollDown
```

### 12.16.4 PTY Integration

The `Pty` type is re-exported from `std/terminal` for convenience:

```
use std/terminal.{Pty, PtyConfig}
```

See [Chapter 10, Section 10.5.2](10-systems.md#1052-pty--pseudo-terminal) for the full `Pty` specification.

---

## 12.17 `std/render`: GPU Rendering

GPU rendering abstractions for terminal and editor applications. See [Chapter 10, Section 10.8](10-systems.md#108-gpu-rendering) for the architecture. This section specifies the module's full API surface.

### 12.17.1 Renderer

```
type Renderer
  doc: "manages the rendering pipeline and backend selection"

fn Renderer.new(config: RenderConfig) -> Result<Renderer, RenderError>
  doc: "initializes the renderer, selecting GPU or CPU backend"
  effects: [unsafe, Gpu]
  panics: never

impl Renderer
  fn draw(self: mut Self, scene: Scene) -> Result<Unit, RenderError> with unsafe, Gpu
  fn resize(self: mut Self, width: UInt32, height: UInt32) -> Result<Unit, RenderError> with unsafe, Gpu
  fn set_font(self: mut Self, font: Font) -> Result<Unit, RenderError> with unsafe, Gpu
  fn backend(self: Self) -> Backend
  fn metrics(self: Self) -> RenderMetrics

struct RenderConfig
  font: FontConfig
  dpi: Float
  vsync: Bool
  backend: Option<Backend>    // None = auto-detect

type Backend
  | Gpu
  | Cpu

struct RenderMetrics
  frame_time: Duration
  draw_calls: Int
  triangles: Int
  glyph_cache_hits: Int
  glyph_cache_misses: Int
```

### 12.17.2 Scene and Surfaces

```
type Scene
  doc: "a retained-mode scene graph describing what to render"

impl Scene
  fn new() -> Scene
  fn add_surface(self: mut Self, surface: Surface) -> SurfaceId
  fn remove_surface(self: mut Self, id: SurfaceId)
  fn get_surface(self: Self, id: SurfaceId) -> Option<Surface>
  fn get_surface_mut(self: mut Self, id: SurfaceId) -> Option<mut Surface>

type Surface
  doc: "a rectangular grid of cells (for terminal rendering)"

impl Surface
  fn new(rows: Int, cols: Int) -> Surface
  fn resize(self: mut Self, rows: Int, cols: Int)
  fn set_cell(self: mut Self, row: Int, col: Int, cell: Cell)
  fn get_cell(self: Self, row: Int, col: Int) -> Cell
  fn clear(self: mut Self)
  fn fill(self: mut Self, cell: Cell)
  fn scroll_up(self: mut Self, lines: Int)
  fn scroll_down(self: mut Self, lines: Int)
  fn rows(self: Self) -> Int
  fn cols(self: Self) -> Int
  fn cursor(self: Self) -> Option<CursorState>
  fn set_cursor(self: mut Self, cursor: Option<CursorState>)
```

### 12.17.3 Cells and Glyphs

```
struct Cell
  char: Char
  fg: Color
  bg: Color
  attrs: CellAttrs

struct CellAttrs
  bold: Bool
  dim: Bool
  italic: Bool
  underline: UnderlineStyle
  strikethrough: Bool
  inverse: Bool
  hidden: Bool

type UnderlineStyle
  | None
  | Single
  | Double
  | Curly
  | Dotted
  | Dashed

type Color
  | Rgb(r: UInt8, g: UInt8, b: UInt8)
  | Rgba(r: UInt8, g: UInt8, b: UInt8, a: UInt8)
  | Indexed(index: UInt8)
  | Default

struct CursorState
  row: Int
  col: Int
  shape: CursorShape
  visible: Bool

type CursorShape
  | Block
  | Underline
  | Bar
  | HollowBlock
```

### 12.17.4 Font Management

```
type Font
  doc: "a loaded font for glyph rasterization"

fn Font.load(config: FontConfig) -> Result<Font, FontError>
  doc: "loads a font from the system or a file path"
  effects: [Fs.read]
  panics: never

struct FontConfig
  family: String
  size: Float
  bold_family: Option<String>
  italic_family: Option<String>
  bold_italic_family: Option<String>

impl Font
  fn metrics(self: Self) -> FontMetrics
  fn rasterize(self: Self, glyph: Char) -> Result<GlyphBitmap, FontError>

struct FontMetrics
  cell_width: Float
  cell_height: Float
  ascent: Float
  descent: Float
  line_height: Float
```

---

## 12.18 `std/signal`: Signal Handling

See [Chapter 10, Section 10.6](10-systems.md#106-signal-handling) for the full specification. This module re-exports the signal types and provides the handler API.

```
use std/signal.{Signal, SignalKind, SignalGuard, SignalStream}
```

---

## 12.19 `std/test`: Testing

Provides the test framework, assertions, and property-based testing.

### 12.19.1 Test Declaration

Tests are functions annotated with `@test`:

```
@test
fn test_parse_valid_json()
  let result = json.parse<User>("{\"name\": \"Alice\", \"age\": 30}")
  assert_ok(result)
  let user = result.unwrap()
  assert_eq(user.name, "Alice")
  assert_eq(user.age, 30)

@test
fn test_parse_invalid_json()
  let result = json.parse<User>("not json")
  assert_err(result)
```

Test files use the `.mn.test` extension and are compiled only during `monel test`.

### 12.19.2 Assertions

```
fn assert(condition: Bool)
fn assert(condition: Bool, message: String)
fn assert_eq<T: Eq + Debug>(left: T, right: T)
fn assert_ne<T: Eq + Debug>(left: T, right: T)
fn assert_ok<T, E: Debug>(result: Result<T, E>)
fn assert_err<T: Debug, E>(result: Result<T, E>)
fn assert_some<T>(option: Option<T>)
fn assert_none<T: Debug>(option: Option<T>)
fn assert_contains(haystack: String, needle: String)
fn assert_starts_with(s: String, prefix: String)
fn assert_ends_with(s: String, suffix: String)
fn assert_approx_eq(left: Float, right: Float, epsilon: Float)
fn assert_panics(f: fn() -> Unit)
fn assert_panics_with(f: fn() -> Unit, message: String)
```

### 12.19.3 Property-Based Testing

```
@test
@property(iterations = 1000)
fn test_sort_preserves_length(input: Vec<Int>)
  let sorted = input.clone().sort()
  assert_eq(sorted.len(), input.len())

@test
@property(iterations = 500)
fn test_parse_roundtrip(user: User)
  let json = json.to_string(user).unwrap()
  let parsed = json.parse::<User>(json).unwrap()
  assert_eq(parsed, user)
```

The `@property` annotation generates random inputs using the `Arbitrary` trait:

```
trait Arbitrary
  fn arbitrary(rng: mut Rng) -> Self
  fn shrink(self: Self) -> Iterator<Item = Self>
```

Monel derives `Arbitrary` automatically for types composed of types that implement `Arbitrary`. All primitive types, `String`, `Vec<T>`, `Option<T>`, and `Result<T, E>` implement `Arbitrary` by default.

### 12.19.4 Contract-Driven Test Generation

The compiler can mechanically generate property-based tests from function contracts. Given:

```
fn push(self: mut Stack<T>, val: T) -> Result<(), StackError>
  ensures:
    ok => self.len == old(self.len) + 1
    ok => self.peek() == Some(val)
    err(Overflow) => self == old(self)
```

Running `monel test --gen-contract-tests` generates and saves to `stack.mn.test`:

```
# auto-generated from contracts for fn push
# regenerate with: monel test --gen-contract-tests

@test
@property(iterations = 1000)
fn contract_push_ok_increments_len(stack: Stack<Int>, val: Int)
  requires: stack.len < stack.capacity
  let old_len = stack.len
  let result = push(stack, val)
  assert_ok(result)
  assert_eq(stack.len, old_len + 1)

@test
@property(iterations = 1000)
fn contract_push_ok_sets_top(stack: Stack<Int>, val: Int)
  requires: stack.len < stack.capacity
  let result = push(stack, val)
  assert_ok(result)
  assert_eq(stack.peek(), Some(val))

@test
@property(iterations = 1000)
fn contract_push_err_overflow_unchanged(stack: Stack<Int>, val: Int)
  requires: stack.len == stack.capacity
  let old_stack = stack.clone()
  let result = push(stack, val)
  assert_err(result)
  assert_eq(stack, old_stack)
```

Each `ensures:` clause becomes a separate test. Conditional postconditions (`ok =>`, `err(Variant) =>`) generate tests with appropriate `requires:` guards that set up the success or failure scenario.

**How it works:**

1. The compiler reads each function's contracts.
2. For each `ensures:` clause, it generates a `@property` test function:
   - `ok => predicate` → test with inputs satisfying all `requires:` and preconditions for the success path
   - `err(Variant) => predicate` → test with inputs that trigger the specific error variant
   - Unconditional `predicate` → test with arbitrary valid inputs
3. `old(expr)` references are captured before the function call.
4. Type invariants generate additional tests verifying the invariant holds after every public method.
5. Generated tests are written to `.mn.test` files and committed to the repository.

Generated tests are committed to the repository and can be reviewed, modified, and versioned like any other code.

**Relationship to SMT verification:**

| Contract verification | Mechanism | Coverage |
|---|---|---|
| SMT (Z3) | Proves contracts hold for **all** inputs | Complete but may timeout on complex properties |
| Generated property tests | Validates contracts for **sampled** inputs | Incomplete but catches implementation bugs SMT misses (e.g., integer overflow at specific values) |

Both run during `monel build`. SMT provides proof; property tests provide empirical validation. They complement each other: SMT catches logical errors, property tests catch implementation bugs (off-by-one, overflow, edge cases in library calls).

### 12.19.5 LLM-Assisted Test Generation

Beyond contract-driven tests, an LLM can generate additional tests that cover scenarios the contracts don't specify:

```
monel test --gen-llm-tests src/auth.mn
```

The LLM reads the function's contracts, implementation, and types, then generates tests for:
- Edge cases the contracts don't cover (empty strings, max-int values, unicode, concurrent access)
- Integration scenarios (sequences of operations that together violate invariants)
- Regression patterns (common bug patterns for the function's effect profile)

Generated tests are saved to `.mn.test` and must be reviewed before committing:

```
monel test --gen-llm-tests src/auth.mn
  Generated 8 tests for fn authenticate
  Generated 3 tests for fn handle_request
  Written to src/auth.mn.test

  Review with: monel diff src/auth.mn.test
```

Generated tests are ordinary code: if the LLM produces a bad test, delete it; if it produces a good one, commit it as a regression guard.

**Pipeline summary:**

```
Contracts (in .mn files)
  │
  ├── SMT verification ──────────── proves for all inputs (compile time)
  ├── Contract-driven prop tests ── validates empirically (test time)
  └── LLM-generated tests ───────── catches what contracts miss (authored once, runs forever)
        │
        └── saved to .mn.test (committed, reviewed)
```

### 12.19.6 Test Fixtures

```
@test
@setup(create_test_db)
@teardown(destroy_test_db)
fn test_user_creation(db: TestDb)
  let user = db.create_user("Alice", "alice@example.com")?
  assert_eq(user.name, "Alice")

fn create_test_db() -> TestDb
  // setup logic

fn destroy_test_db(db: TestDb)
  // teardown logic
```

### 12.19.7 Test Utilities

```
type MockServer
  # HTTP mock server for testing

fn MockServer.new() -> MockServer
fn MockServer.mock(self: mut Self, method: Method, path: String, response: Response) -> mut Self
fn MockServer.url(self: Self) -> String
fn MockServer.assert_called(self: Self, method: Method, path: String, times: Int)

fn with_temp_dir(f: fn(String) -> Unit) -> Unit
  effects: [Fs.write]

fn with_env_var(name: String, value: String, f: fn() -> Unit) -> Unit
  effects: [Env.read, Env.write]
```

---

## 12.20 `std/log`: Logging

Structured logging with level filtering.

```
type LogLevel
  | Trace
  | Debug
  | Info
  | Warn
  | Error

fn log(level: LogLevel, message: String, fields: Map<String, String>)
  doc: "emits a structured log entry"
  effects: [Log]

// Convenience macros/functions
fn trace(message: String)
fn debug(message: String)
fn info(message: String)
fn warn(message: String)
fn error(message: String)

// Structured variants
fn info_with(message: String, fields: Map<String, String>)
fn error_with(message: String, fields: Map<String, String>)

fn set_level(level: LogLevel)
  doc: "sets the minimum log level for output"
  effects: [Log]

fn set_output(writer: impl Write)
  doc: "sets the log output destination"
  effects: [Log]

fn set_format(formatter: fn(LogEntry) -> String)
  doc: "sets a custom log format function"
  effects: [Log]

struct LogEntry
  level: LogLevel
  message: String
  fields: Map<String, String>
  timestamp: Instant
  module: String
  file: String
  line: Int
```

---

## 12.21 `std/fmt`: Formatting

Formatting traits and utilities.

### 12.21.1 Display and Debug

```
trait Display
  doc: "human-readable text representation of a value"
  fn fmt(self: Self, f: mut Formatter) -> Result<Unit, FmtError>

trait Debug
  doc: "debug/programmer-oriented text representation of a value"
  fn fmt(self: Self, f: mut Formatter) -> Result<Unit, FmtError>
```

`Display` is used by string interpolation (`"hello, {name}"`) and `println`. `Debug` is used by `{:?}` format specifier and `dbg!`.

### 12.21.2 Formatter

```
struct Formatter
  fn write_str(self: mut Self, s: String) -> Result<Unit, FmtError>
  fn write_char(self: mut Self, c: Char) -> Result<Unit, FmtError>
  fn precision(self: Self) -> Option<Int>
  fn width(self: Self) -> Option<Int>
  fn fill(self: Self) -> Char
  fn align(self: Self) -> Option<Alignment>
  fn sign_plus(self: Self) -> Bool
  fn sign_minus(self: Self) -> Bool
  fn alternate(self: Self) -> Bool     // # flag

type Alignment
  | Left
  | Right
  | Center
```

### 12.21.3 Derive

`@derive(Debug)` and `@derive(Display)` generate implementations automatically:

```
@derive(Debug)
struct Point
  x: Float
  y: Float

// Debug output: Point { x: 1.0, y: 2.0 }
```

`@derive(Display)` for enums uses variant names. For structs, a custom format string is required:

```
@derive(Display("{x}, {y}"))
struct Point
  x: Float
  y: Float

// Display output: 1.0, 2.0
```

---

## 12.22 `std/iter`: Iterators

Iterator traits and combinators.

### 12.22.1 Core Trait

```
trait Iterator
  doc: "a sequence of values that can be consumed one at a time"
  type Item
  fn next(self: mut Self) -> Option<Self.Item>

  // Provided methods (with default implementations):
  fn map<U>(self, f: fn(Self.Item) -> U) -> Map<Self, U>
  fn filter(self, f: fn(Self.Item) -> Bool) -> Filter<Self>
  fn filter_map<U>(self, f: fn(Self.Item) -> Option<U>) -> FilterMap<Self, U>
  fn flat_map<U, I: Iterator<Item = U>>(self, f: fn(Self.Item) -> I) -> FlatMap<Self, I>
  fn flatten(self) -> Flatten<Self>  // where Item: Iterator
  fn enumerate(self) -> Enumerate<Self>
  fn zip<U: Iterator>(self, other: U) -> Zip<Self, U>
  fn chain(self, other: Self) -> Chain<Self>
  fn take(self, n: Int) -> Take<Self>
  fn skip(self, n: Int) -> Skip<Self>
  fn take_while(self, f: fn(Self.Item) -> Bool) -> TakeWhile<Self>
  fn skip_while(self, f: fn(Self.Item) -> Bool) -> SkipWhile<Self>
  fn peekable(self) -> Peekable<Self>
  fn inspect(self, f: fn(Self.Item) -> Unit) -> Inspect<Self>
  fn step_by(self, step: Int) -> StepBy<Self>
  fn chunks(self, size: Int) -> Chunks<Self>
  fn windows(self, size: Int) -> Windows<Self>

  // Terminal operations:
  fn collect<C: FromIterator<Self.Item>>(self) -> C
  fn fold<A>(self, init: A, f: fn(A, Self.Item) -> A) -> A
  fn reduce(self, f: fn(Self.Item, Self.Item) -> Self.Item) -> Option<Self.Item>
  fn for_each(self, f: fn(Self.Item) -> Unit)
  fn count(self) -> Int
  fn sum(self) -> Self.Item         // where Item: Add
  fn product(self) -> Self.Item     // where Item: Mul
  fn min(self) -> Option<Self.Item> // where Item: Ord
  fn max(self) -> Option<Self.Item> // where Item: Ord
  fn min_by(self, f: fn(Self.Item, Self.Item) -> Ordering) -> Option<Self.Item>
  fn max_by(self, f: fn(Self.Item, Self.Item) -> Ordering) -> Option<Self.Item>
  fn find(self, f: fn(Self.Item) -> Bool) -> Option<Self.Item>
  fn position(self, f: fn(Self.Item) -> Bool) -> Option<Int>
  fn any(self, f: fn(Self.Item) -> Bool) -> Bool
  fn all(self, f: fn(Self.Item) -> Bool) -> Bool
  fn nth(self, n: Int) -> Option<Self.Item>
  fn last(self) -> Option<Self.Item>
  fn join(self, separator: String) -> String  // where Item: Display
```

### 12.22.2 IntoIterator

```
trait IntoIterator
  type Item
  type IntoIter: Iterator<Item = Self.Item>
  fn into_iter(self) -> Self.IntoIter
```

Any type implementing `IntoIterator` can be used in `for` loops:

```
for item in collection
  // collection.into_iter() is called implicitly
```

### 12.22.3 FromIterator

```
trait FromIterator<T>
  fn from_iter(iter: impl Iterator<Item = T>) -> Self
```

Enables `.collect()` to produce any collection type:

```
let vec: Vec<Int> = (0..10).filter(|x| x % 2 == 0).collect()
let set: Set<Int> = (0..10).collect()
let map: Map<String, Int> = entries.iter().map(|e| (e.name.clone(), e.value)).collect()
```

---

## 12.23 `std/mem`: Memory Utilities

Low-level memory utilities. Most operations require the `unsafe` effect.

```
fn size_of<T>() -> Int
  doc: "returns the size of type T in bytes"
  effects: []

fn align_of<T>() -> Int
  doc: "returns the alignment of type T in bytes"
  effects: []

fn swap<T>(a: mut T, b: mut T)
  doc: "swaps the values at two mutable references"
  effects: []

fn replace<T>(dest: mut T, value: T) -> T
  doc: "replaces the value at dest, returning the old value"
  effects: []

fn take<T: Default>(dest: mut T) -> T
  doc: "takes the value at dest, leaving Default::default() in its place"
  effects: []

fn drop<T>(value: T)
  doc: "explicitly drops a value, running its destructor"
  effects: []

fn forget<T>(value: T)
  doc: "prevents a value from being dropped (leaks resources)"
  effects: []
```

### 12.23.1 Arena Allocator

```
type Arena
  doc: "a bump allocator that frees all memory at once"

impl Arena
  fn new(initial_capacity: Int) -> Arena
  fn alloc<T>(self: Self, value: T) -> T
  fn alloc_slice<T>(self: Self, values: Array<T>) -> Array<T>
  fn alloc_zeroed<T>(self: Self, count: Int) -> mut Array<T>
  fn bytes_allocated(self: Self) -> Int
  fn reset(self: mut Self)   // frees all allocations, reuses backing memory

impl Drop for Arena
```

```
type TypedArena<T>
  doc: "a bump allocator for values of a single type"

impl TypedArena<T>
  fn new() -> TypedArena<T>
  fn alloc(self: Self, value: T) -> T
  fn alloc_many(self: Self, values: impl Iterator<Item = T>) -> Array<T>
  fn iter(self: Self) -> Iterator<Item = T>
  fn len(self: Self) -> Int
```

---

## 12.24 `std/ffi`: Foreign Function Interface

FFI helpers and C type definitions. See [Chapter 10, Section 10.4](10-systems.md#104-foreign-function-interface-ffi) for the full FFI specification.

### 12.24.1 C String Types

```
type CStr
  doc: "a borrowed null-terminated C string"

impl CStr
  fn from_ptr(ptr: Ptr<CChar>) -> CStr with unsafe
  fn as_ptr(self: Self) -> Ptr<CChar>
  fn to_str(self: Self) -> Result<String, Utf8Error>
  fn to_string_lossy(self: Self) -> String
  fn len(self: Self) -> Int

type CString
  doc: "an owned null-terminated C string"

impl CString
  fn new(s: String) -> Result<CString, NulError>
  fn from_raw(ptr: MutPtr<CChar>) -> CString with unsafe
  fn as_c_str(self: Self) -> CStr
  fn as_ptr(self: Self) -> Ptr<CChar>
  fn into_raw(self) -> MutPtr<CChar>
```

### 12.24.2 C Type Aliases

All C-compatible type aliases are defined here and re-exported. See the type mapping table in [Chapter 10, Section 10.4.2](10-systems.md#1042-c-type-mappings).

---

## 12.25 Key Traits Summary

This section collects all key traits for reference.

### 12.25.1 Equality and Ordering

```
trait Eq
  fn eq(self: Self, other: Self) -> Bool
  fn ne(self: Self, other: Self) -> Bool  // default: !self.eq(other)

trait Ord: Eq
  fn cmp(self: Self, other: Self) -> Ordering
  fn lt(self: Self, other: Self) -> Bool   // default via cmp
  fn le(self: Self, other: Self) -> Bool   // default via cmp
  fn gt(self: Self, other: Self) -> Bool   // default via cmp
  fn ge(self: Self, other: Self) -> Bool   // default via cmp

type Ordering
  | Less
  | Equal
  | Greater
```

### 12.25.2 Hashing

```
trait Hash
  fn hash(self: Self, hasher: mut impl Hasher)
```

### 12.25.3 Cloning and Copying

```
trait Clone
  fn clone(self: Self) -> Self

trait Copy: Clone
  // marker trait -- values are copied on assignment rather than moved
```

`Copy` is implemented by all primitive types, `Ptr<T>`, and `MutPtr<T>`. Types containing heap allocations (e.g., `String`, `Vec<T>`) implement `Clone` but not `Copy`.

### 12.25.4 Destruction

```
trait Drop
  fn drop(self: mut Self)
```

### 12.25.5 Conversion

```
trait From<T>
  fn from(value: T) -> Self

trait Into<T>
  fn into(self) -> T
```

`Into<T>` is automatically implemented for any type that implements `From<T>`. The blanket implementation is:

```
impl<T, U> Into<U> for T where U: From<T>
  fn into(self) -> U
    U.from(self)
```

### 12.25.6 Default Values

```
trait Default
  fn default() -> Self
```

### 12.25.7 Serialization

```
trait Serialize
  fn serialize(self: Self, serializer: mut Serializer) -> Result<Unit, SerializeError>

trait Deserialize
  fn deserialize(deserializer: mut Deserializer) -> Result<Self, DeserializeError>
```

These traits are used by `std/json` and can be used by other serialization formats. `@derive(Serialize, Deserialize)` generates implementations automatically.

### 12.25.8 Display and Debug

```
trait Display
  fn fmt(self: Self, f: mut Formatter) -> Result<Unit, FmtError>

trait Debug
  fn fmt(self: Self, f: mut Formatter) -> Result<Unit, FmtError>
```

### 12.25.9 Iterator

```
trait Iterator
  type Item
  fn next(self: mut Self) -> Option<Self.Item>
```

### 12.25.10 Derive Summary

The following traits can be automatically derived with `@derive(...)`:

| Trait | Derives For | Behavior |
|-------|-------------|----------|
| `Eq` | Structs, enums | Field-by-field / variant-by-variant equality |
| `Ord` | Structs, enums | Lexicographic ordering by fields / variant order |
| `Hash` | Structs, enums | Combines field hashes |
| `Clone` | Structs, enums | Clones each field |
| `Copy` | Structs (all fields Copy) | Marker only |
| `Debug` | Structs, enums | `TypeName { field: value, ... }` format |
| `Display` | Enums (variant names), structs (with format string) | Human-readable |
| `Default` | Structs (all fields Default) | All fields default |
| `Serialize` | Structs, enums | JSON-compatible serialization |
| `Deserialize` | Structs, enums | JSON-compatible deserialization |
| `Arbitrary` | Structs, enums (all fields Arbitrary) | Random generation for property tests |

---

## 12.26 Module Dependency Graph

The standard library modules have the following dependency structure. Arrows indicate "depends on":

```
std/ffi ─────────── std/mem
    │                  │
    v                  v
std/io ◄──────── std/fs
    │                │
    v                v
std/net ────────► std/async ◄──── std/sync
    │                │
    v                v
std/http         std/signal
                     │
std/json             v
    │            std/terminal ──► std/render
    v                │
std/text             v
                 std/process
std/math
std/time
std/crypto
std/env
std/log
std/fmt
std/iter
std/test
std/collections
```

Core modules (`std/io`, `std/mem`, `std/fmt`, `std/iter`) have no dependencies beyond the language primitives. Higher-level modules (`std/http`, `std/terminal`, `std/render`) build on lower-level ones.

Every module in the standard library:
- Has its own contracts (in `.mn` files).
- Is subject to parity verification.
- Can be independently versioned (though standard library versions track the compiler).
- Can be replaced by user code (no special compiler privileges).
