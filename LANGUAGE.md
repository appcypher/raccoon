## POSSIBLE ADDITIONS

- Multiline lambda expression

  ```py
  map(
      def (x):
          if x == 1: 0
          else: 5
      array
  )

  map(
      (def (x):
          if x == 1: 0
          else: 5
      ),
      array
  )

  map((
      def (x):
          if x == 1: 0
          else: 5
  ), array)
  ```

- Implicit returns

  ```py
  def foo(x):
      if x == 1: 0
      else: 5

  assert_eq(foo(1), 0)
  assert_eq(foo(2), 5)
  ```

- Updated type annotation & generics [In Progress]

  Raccoon's types are not erased, so they are available at runtime.

  ```py
  # Type anotation
  index: int = 9

  # List type
  nums: [int] = []

  # Array type
  nums: [int, 3] = [1, 2, 3]
  nums = @[1, 2, 3] # alternative syntax

  # bytes type
  name: bytes = b"John Doe"
  name = @b"John Doe" # alternative syntax

  # String type
  name: String = "John Doe"

  # str type
  name: str = "John Doe"
  name = @"John Doe" # alternative syntax

  # Char type
  letter: char = 'a'

  # Dictionary type
  nums: {int, String} = {1: "one", 2: "two"}
  nums = {}

  # Set type
  nums: {int} = {1, 2, 3}
  nums = @{} # alternative syntax

  # Tuple type
  value: (int, int) = (1, 2, 3)
  value: (int, *String) = (1, "hello", "world")

  # Optional type
  age: int? = 45

  # Result type
  age: int! = get_age()

  # Function type
  fn: (int, int) -> int = sum
  fn: Fn[(int, int), int] = sum
  fn: int -> int = square

  # Intersection type
  identity: int & str = "XNY7V40"

  # Type comparison
  Dog < Pet
  Pet > Dog
  Dog == Dog

  # Generics
  @impl(Sequence).where(N: int)
  enum class TinyList[T, const N]:
      Inline(t: [T, N])
      Heap(t: [T])
  ```

  You can omit the type of a function

  ```py
  def foo(x):
      x + 1

  y = foo(1)
  ```

  But when you specify the argument types, you must also specify the return type

  ```py
  def foo(x: int) -> int:
      x + 1

  y = foo(1)
  ```

- Untyped classes

  Untyped classes are classes with one or more untyped fields.

  They are treated like generic classes in that their instances may not be compatible depending on the inferred specific type.

  ```py
  class SockAddr:
      def init(self, addr, port):
          self.addr = addr
          self.port = port
  ```

  ```py
  addr1 = SockAddr("127.0.0.1", 8080) # SockAddr[str, int]
  addr2 = SockAddr([127, 0, 0, 1], 8080) # SockAddr[[int, 4], int]

  addr == addr2 # Error. Types are not compatible
  ```

- Checking types

  ```py
  data class SockAddr(addr, port)

  addr = SockAddr("127.0.0.1", 8080)
  ```

  `type` function returns the generic type of an object

  ```py
  addr.type() # SockAddr
  ```

  `type_concrete` function returns the concrete type of an object

  ```py
  addr.type_concrete() # SockAddr[str, int]
  ```

- More operators

  ```py
  @where(T: Number)
  class Num[T]:
      def init(self, value: T):
          self.value = value

      def plus(self, other):
          Num(self.value + other.value)

      def sqrt(self):
          Num(√(self.value))

      def square(self):
          Num(self.value²)

  a, b = Num(2), Num(3)

  sum = a + b
  rooted = √a
  squared = a²
  ```

- Pow and xor operators

  ```py
  pow = 2 ^ 10
  ```

  ```py
  xor = 2 || 10
  ```

- Macro metaprogramming [In Progress]

  Raccoon's decorators are macros. They are typed and they are hygenic.

  ```py
  @macro
  def test(path: Path, attr: Attr[MyClass]) -> Tree:
      # ...
      pass

  @macro
  def test(path: Path, block: Block[Tree]) -> Tree:
      # ...
      pass

  @macro
  def test(path: Path, attr: Attr[MyClass], block: Block[Tokens[MyClassOther]]) -> Tree:
      # ...
      pass

  @macro
  def test(path: Path, attr: Attr[MyClass], fn: Fn) -> Tree:
      Fn {
          attrs,
          vis,
          name,
          sig,
          body,
      } = tree.value

      args = sig.args
      return_type = sig.return_type
      renamed = @format_ident(f"__mangled_{name}")

      @quote:
          $( attrs )*
          $vis def $name ( $( args )* ) -> $return_type:
              $body

          $vis def $renamed ( $( args )* ) -> $return_type:
              $body
  ```

  There are multiple ways of using a macro depending on the type of the argument.

  ```py
  @test("Hello world!")

  @test:
      print("Hello world!")

  @test(first):
      print("Hello world!")

  @test(info)
  def foo():
      print("Hello world!")
  ```

  You can also chain macros

  ```py
  @base(Pet).impl(Debug)
  pub data class Dog(name: str, age: int):
      def debug(self, fmt):
          fmt.debug_class("Dog")
              .field("name", self.name)
              .field("age", self.age)
              .finish()
  ```

- Function declaration syntax

  ```py
  def add(a: int, b: int) -> int
  def add(int, int) -> int
  def display(str)
  ```

- Type alias

  ```py
  type IdentityFunc[T] = T -> T
  ```

- Character

  ```py
  ch0 = 'a'
  ch1 = '\n'
  ch2 = '\uff'
  ch3 = '\890'

  string = ch0 + ch1

  if 'a' <= ch <= 'z':
      print(ch)
  ```

- Integer literals

  ```py
  value: uint = 0x1234
  value: u8 = 0o1234_u8
  value: u16 = 0b1010_1010_1010_1010_u16

  value: int = 0x1234
  value: i8 = 0o1234_i8
  value: i16 = 0b1010_1010_1010_1010_i16
  ```

- Regex literal [In Progress]

  ```js
  regex = /\d+/;
  ```

  Regex literal with `/.../` syntax is notoriously hard to lex.
  In Racoon's case we need to make sure no expression-type token comes before it.
  Although this makes the lexer more complicated.

- The no_wrap macro

  You can define your enum so that a particular variant does not need to be wrapped to be passed as a value to the enum.
  This only works for variants with a single field and can only be applied to one variant under an enum.
  This is how the `Option` and `Result` are defined.

  ```py
  pub enum class Option[T]:
      @no_wrap
      Some(t: T)
      None
  ```

  So we don't have to wrap `result` in `Some` here

  ```py
  def get_age(self) -> u8!:
      result = self.some_calc()
      if result < 0:
          raise Error("Age cannot be negative")
      result
  ```

  The same applies here

  ```py
  mut age: Option[u8] = 0
  ```

- `?` operator

  ```py
  def get_surname(p: Person) -> String?:
      (_, _, lastname) = p.get_names()?
      lastname
  ```

- `!` operator

  ```py
  def fetch_peer_addr(net: Network, id: [u8; 16]) -> IpAddr!:
      Peer { addr, .. } = net.fetch_peer(id)!
      addr
  ```

- List and array type annotation

  List

  ```py
  nums: List[int] = [1, 2, 3, 4] # or
  nums: [int] = [1, 2, 3, 4]
  ```

  Sized array

  ```py
  nums: Array[int, 4] = [1, 2, 3, 4] # or
  nums: [int, 4] = [1, 2, 3, 4]
  nums = @[1, 2, 3, 4]
  ```

  Unsized array

  ```py
  nums: Array[u8, *] = alloc.allocate_zeroed(Layout(10, 8)!)! # or
  nums: [u8, *] = alloc.allocate_zeroed(Layout(10, 8)!)!
  ```

- Additional reserved keywords

  ```py
  Self, union, enum, dyn, new, abstract, data, const, ref, ptr, val, match, let, mut, var, where, macro, type, pub
  ```

- `mut` keyword

  Raccoon variables are immutable by default unless explicitly made mutable with the `mut` keyword.

  ```py
  def map(array, f):
      mut t = []
      for i in array:
          t.append(f(i))
      return t
  ```

- New versions of certain functions and objects by default

  ```py
  map, sum, filter, sum, join

  map(array, def (x): x + 1)
  ```

- Explicit reference

  ```rust
  num2 = ref num
  num3 = num2
  ```

- Pointers

  ```nim
  num2 = ptr num
  num2 += 1
  num3 = val num2
  ```

- New named tuple syntax

  ```py
  named_tup = (name = "James", age = 10)
  named_tup.name
  ```

  Named tuple is still position-based when unpacking

  ```py
  (name, age) = named_tup
  ```

- Introducing more primitive types

  ```py
  u8, u16, u32, u64, usize
  i8, i16, i32, i64, isize
  f32, f64
  ```

- Updated match statement

  Does exhaustiveness checks.

  ```py
  match x:
      case Class { x, .. }: x
      case DataClass (x, **z) : x # (positional)
      case EnumVariant(x, **z): x # (positional)
      case EnumVariant: 0
      case [x, 5 as y, z]: y # List (positional)
      case (x, 5 as y, 10, *z): x # Tuple (positional)
      case (x, 5 as y, 10, **z): x # NamedTuple (positional)
      case { "x", "y" as y,  *z}: x # Set
      case { "x", "y" as y, **z}: x # Dict
      case 10 | 11 as x: x
      case 0..89: 10
      case other: other
      case _: 0
  ```

- `if-let` statement

  ```py
  if let Variant { name, age } = x:
      print(name)
  else:
      print("Not a person")
  ```

- `let-else` statement

  ```py
  let Variant { name, age } = x else:
      raise Error("Not a person")

  print(name)
  ```

- `if-else` expression

  ```py
  x = if cond():
      10
  else:
      20
  ```

  ```py
  x = if cond(): 10
  else: 20
  ```

  ```py
  x = if cond: 10 else: 20
  ```

- Partial application

  ```py
  add2 = add(2, _)
  add10 = add(_, 10)
  ```

- Overloading functions and methods based on arity and types

  ```py
  def add(a: String, b: String):
      a + '_' + b

  def add(a: int, b: int):
      a + b

  def add(a, b, c):
      return a + b + c

  add("Hello", "world")
  add(1, 2)
  add(1, 2, 3)
  ```

- Updated range and index range syntax

  ```py
  ls = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
  print(ls[1..3]) # [2, 3, 4]
  ```

  ```py
  for i in 0..10:
      print(i)
  ```

  ```py
  for i in 1..=10:
      print(i)
  ```

- Generator

  A generator is implements the `Iterator` trait.

  ```py
  def get_data(max):
      for i in 0..max:
          yield i

  for x in get_data():
      print(x)
  ```

- Async functions and blocks

  An async function or block implements the `Future` trait.

  ```py
  async def fetch_num() -> int:
      await sleep(1)
      return 10

  def fetch_num() -> Future[int]:
      async:
        await sleep(1)
        return 10
  ```

- Async generator and `for-await` statement

  An async generator implements the `AsyncIterator` trait.

  ```py
  async def stream_data(max):
      for i in 0..max:
          yield await get_data(i)

  for await x in stream_data():
      print(x)
  ```

- Underscore means discarded value or unprovided value

  ```py
  for _ in 1..11:
      print("ping")
  ```

  ```py
  partial_add = add(1, _)
  partial_add(2)
  ```

- Support for partial application

  ```py
  def add(a, b):
      a + b

  partial_add = add(1, _)
  partial_add(2)
  ```

- Updated abstract class syntax

  Abstract classes can't have `init` contructors.

  ```py
  @where(S: Subscription)
  abstract class Observable[S]:
      subscriptions: [S]

      def notify(message: String)
      def add_subscription(sub: S)
  ```

  ```py
  @where(S: Subscription)
  abstract class Observable[S](subscriptions: [S]):
      def notify(message: String)
      def add_subscription(sub: S)
  ```

- No need for associated type

  Raccoon does not have associated types because it deliberately reduces the type of abstraction the implementing type can have which I find unnecessary.

  ```py
  abstract class Iter[T]:
      def next(mut self) -> Option[T]

  @impl(Iter[uint]) # `@impl(Iter)` also works because the generics can be inferred
  data class Numbers(length: uint, mut item: uint = 0):
      def next(mut self) -> uint?:
          if self.item >= self.length:
              return None
          self.item += 1
          self.item
  ```

  Let's you express more abstract types.

  ```py
  @impl(Iter[T]).where(T: Ord, Eq, .inc(T))
  data class Numbers[T](length: T, item: T = default()):
      def next(mut self) -> T?:
          if self.item >= self.length:
              return None
          self.item.inc()
          self.item
  ```

- Update enum syntax

  ```py
  enum class Data:
      Inline(val: [u8])
      External(addr: String, port: u16)
  ```

  ```py
  enum class Option[T]:
      @no_wrap
      Some(t: T)
      None

  identity: Option[String] = Option.None

  match identity:
      case Some(value): value
      case _: None
  ```

- Updated data class syntax

  Maybe look for another keyword

  ```py
  data class Person(name, age, gender):
      def debug(self, fmt):
          fmt.debug_data_class("Person")
            .field("name", self.name)
            .field("age", self.age)
            .field("gender", self.name)
            .finish()

  data class Person(name: String, age: int, gender=Gender.Female)

  john = Person("John", 50)
  id = Identity(504)
  ```

- No magic method syntax

  ```py
  class Person:
      def init(self, name, age):
          self.name = name
          self.age = age
  ```

- Updated import syntax

  ```py
  import std.collections.LinkedList
  import std.collections.Deque
  import std.result.Result
  import anyhow.Result as AnyResult
  ```

  ```py
  import std.{
    collections.{ LinkedList, Deque}
    result.Result
  }
  import anyhow.Result as AnyResult
  ```

  ```py
  import std.collections.*
  import std.result.{ self, * }
  ```

  Unlike Python and like Rust the abstract methods that a class implements have to be explicitly imported.

  ```py
  import std.futures.Stream
  import gaze.{RemoteNumbers, Type}

  for await i in RemoteNumbers(Type::Pi).get_stream():
      print(i)
  ```

- `Gaze` is to Raccoon as `crate` is to Rust

- Module items are private by default

  Visibility outside the gaze

  ```py
  pub data class Person(pub name, pub age)
  ```

  Visibility inside the gaze

  ```py
  pub(gaze) data class Person(name, age)
  ```

- No class field. Only instance field definition

  ```py
  pub class Person:
      pub name: String
      pub age: int
  ```

- Unified function call syntax

  ```py
  data class Person(name, age):
    def get_name(p: Person) -> String:
        p.name

  def greet(self, name):
      print(f"Hello, {name}!")

  p = Person("John", 50)
  ```

  ```py
  get_name(p)
  p.get_name()
  ```

  ```py
  greet(p)
  p.greet()
  ```

- Unlike Python, instance cannot call class methods with the dot notation.

  ```py
  class TestSetup:
      def initialize():
          # ...
          pass

  t = Test()
  TestSetup.initialize() # Okay
  t.initialize() # Compile error
  ```

- Global items

  ```py
  @global_allocator
  global GLOBAL_ALLOC: TinyAlloc = default()
  ```

  ```py
  @global_allocator
  global GLOBAL_RUNTIME: Futuro = default()
  ```

- Return type polymorphism

  ```py
  @impl(Default)
  class String:
      def default(): ""

  @impl(Default)
  class int:
      def default(): 0

  def default[T]() -> T:
      T.default()

  name: String = default()
  age: int = default()
  ```

- Higher kinded types [In Progress]

  ```py
  abstract class Functor[F[_]]:
      def map[A, B](self, f: A -> B, fa: F[A]) -> F[B]

  @impl(Functor[Option])
  class OptionFunctor:
      def map[A, B](self, f: A -> B, fa: Option[A]) -> Option[B]:
          match fa:
              case Some(value): f(value)
              case _: None
  ```

- Effects system [In Progress]

  ```py

  ```
