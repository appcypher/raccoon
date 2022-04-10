# Interface Contract

The body of an untyped function defines its `interface contract`.

Observing the following example:

```py
def add(a, b):
    return a + b
```

`add` has the following interface contract:

    [ T: impl __plus__.2 ](a: any T.__plus__.0, b: any T.__plus__.1)

- `[ T: impl __plus__.2 ]` reads as:

  T is a type that implements `__plus__` method that takes 2 arguments

- `(a: any T.__plus__.0, b: any T.__plus__.1)` reads as:

  a is the reference or value of some type that can be passed as first argument to method `T.__plus__`.

  b is the reference or value of some type that can be passed as first argument to method `T.__plus__`.

- `any` represents `ref` or `val` of the type.

When we then call `add`, given that the arguments satisfy the interface contract, we instantiate a concrete `add` at compile-time.

```py
x = add(1, 2)
```

`add` as used above has the instantiation `def add(int, int) -> int`.

The above illustration is an example of an `argument contract`. The arguments of `add` must **have types that can appear in certain positions of the `__plus__` function**.

There is also a `return contract` that an instantiation may need to satisfy.

Say we have an abstract class with a method that allows the implementor to return any type.

```py
abstract class Giver:
    abstract def gift(self)

@implements(Giver)
class StringGiver:
    def gift(self) -> str:
        return "string gift" # Has an str return type

@implements(Giver)
class IntGiver:
    def gift(self) -> int:
        return 8080 # Has an int return type
```

The abstract class can then be used as part of a function's `interface contract`.

```py
def iterate_gift(giver: Giver):
    for gift in giver.gift():
        print(f"{gift}")

iterate_gift(StringGiver()) # Okay because type returned by StringGiver.gift `str` implements __iter__ and __next__ which implements __str__ and so on.
iterate_gift(IntGiver()) # Error because type returned by IntGiver.gift `int` does not implement __iter__ and __next__.
```

Notice that `IntGiver` won't work with `iterate_gift` because while it satisfies its `argument contract`, it does not satisfy its `return contract`.

The `interface contract` of `iterate_gift` looks like this.

    [
        T: impl gift.1,
        U: impl __iter__.1,
        V: impl __next__.1,
        W: impl __str__.1,
    ](
        giver: any T.0,
        @where [                                  // a is something that implements T at pos 0
            T.gift returns any U.__iter__.0,     // T.gift returns something that implements U at pos 0
            U.__iter__ returns any V.__next__.0, // U.__iter__ returns something that implements V at pos 0
            V.__next__ returns any W.__str__.0,  // V.__next__ returns something that implements V at pos 0
        ]
        @returns W.__str__
    )

```py
def iterate_gift(giver: Giver):
    return iter(giver.gift()).next()

s = iterate_gift(StringGiver()) # s has type `str` from the instantiation of iterate_gift.
```

Here `iterate_gift` has the following `interface contract`:

    [
        T: impl gift.1,
        U: impl __iter__.1,
        V: impl __next__.1,
    ](
        a: any T.0,
        @where [                                  // a is something that implements T at pos 0
            T.gift returns any U.__iter__.0,     // T returns something that implements U at pos 0
            U.__iter__ returns any V.__next__.0, // U returns something that implements V at pos 0
        ]
        @returns V.__next__
    )

`iterate_gift` as used above has the instantiation `iterate_gift(void) -> str`. `void` because `StringGiver` has no field, so no space is allocated for it.

As mentioned before, what you do with the arguments of an untyped function determines the `interface contract` and the kind of monormorphisation allowed.

# Common Fields

We have seen above that the methods you use with an argument determine the interface contract of an untyped function. This is also true for the fields of an argument.

```py
def who_am_i(something):
    print(f"I am {something.name}")
```

`who_am_i` will only work for arguments that have a name method. The interface contract of `who_am_i` looks like this:

    [
        T: impl __str__.1
    ](
        something: any {
            name: T.__str__.0
        },
        @returns T.__str__
    )

Notice the `impl [name]` syntax. It used to refer to a field. `impl func.x` is used for methods.

Also notice that we use `T: any U.0` syntax for fields just like arguments because they are values that must also conform to some implementation.

# Additional Example

```py
class Point:
    def __init__(self, x, y):
        self.x = x
        self.y = y

    def __plus__(self, other):
        return Point(self.x + other.x, self.y + other.y)

def add(a, b):
    return a + b

point1 = Point(1, 2)
point2 = Point(3, 4)

print(point1 + point2)
```

    add: [T: impl __plus__] (
        self: T.__plus__.0,
        other: T.__plus__.1,
        @returns T.__plus__
    )

    __plus__: [
        T: impl __plus__.2,
        U: impl __init__Point.2,
    ] (
        self: {
            x: T.__plus__.0,
            y: T.__plus__.0,
        },
        other: {
            x: T.__plus__.1,
            y: T.__plus__.1,
        },
        @where [
            T.__plus__ returns U.__init__Point.0,
            T.__plus__ returns U.__init__Point.1,
        ],
        @returns U.__init__Point
    )

    __init__Point: [ T: __new__.2 ] (
        x: T.__new__.0,
        y: T.__new__.1,
        @returns T.__new__
    )

    let point1 { x: usize, y: usize } = __init__Point#1(1, 2) // Object Instantiation. __init__Point#1(usize, usize) an instantiation made here.
    let point2 { x: usize, y: usize } = __init__Point#1(3, 4) // Object Instantiation

    let tmp = __plus__#1(point1, point2) // Object Instantiation. __plus__#1(Point, Point) an instantiation made here.

# Generics

Generics are useful for restricting an interface contract further because it allows certain conditional semantics that a developer may desire.

```py
@where(T: < Seq, U: < Seq) # Reads as where T is a subtype of Seq. Speculative syntax and type.
def any_common_elements(l: T, r: U) -> bool:
    for (a, b) in zip(a, b):
        if a == b:
            return true

    return false

any_common_elements([1, 2, 3], {4, 5, 3}) # true
```

# Intersection Types and Type Safety

Raccoon handles type safety differently. For example, when a function can return multiple types at runtime, Raccoon returns an intersection of both types.

```py
def unsafe():
    if cond():
        return StringGiver().gift()
    else:
        return IntGiver().gift()

t = unsafe()
```

`unsafe` as used above has the instantiation `def unsafe() -> int & str`.

`int & str` is implemented as a [tagged union](https://en.wikipedia.org/wiki/Tagged_union). On the other hand `dyn` objects are implemented as `[tag, ..fields]`in memory.

`int` and `str` are variants of `int & str`.

Intersection types are similar to `dyn AbstractClass`, except that they are used in places where the compiler can easily determine number of types that make up the intersection. For example, enum variants, unsafe return types, etc.

```py
x: int & str = get_int_or_str()
double = x + x
```

In places where there can be potentially many types, we use dynamic dispatch via `dyn` objects. For example, container types.

Raccoon defines `intersection types` differently from other languages like [Crystal](https://crystal-lang.org/reference/1.3/syntax_and_semantics/union_types.html), [TypeScript](https://www.typescriptlang.org/docs/handbook/unions-and-intersections.html), [Julia](https://docs.julialang.org/en/v1/manual/types/#Type-Unions) or [Pony](https://tutorial.ponylang.io/types/type-expressions.html?h=inter#unions).

Raccoon uses it from an implementation (or member) perspective rather than a shape perspective. So type intersection in Raccoon means the `implementations at the intersection` of the combined types implementations.

`&` is used to represent the idea just like in set theory.

```py
Ob1000 & Ob1100 == Ob1000

{ foo, bar, qux } & { tee, foo } == { foo }
```

https://stackoverflow.com/questions/59722333/union-and-intersection-of-types/59723040#59723040

Raccoon's intersection types is position-independent. This behavior is naturally expected of method members but it also applies to field members.

```py
data class A(x: int, y: str)
data class B(w: str, x: int)
data class C(x: int)
```

This means type `C` can pass where `A & B` is expected even though `B` has its `x` field in a different position from a data layout perspective.

# Union Classes

Union classes are analogous to C union types. Their memory layout is determined by their largest field.

```py
union class JSNumber:
    int: i64
    float: f64

JSNumber.float = 2.0
print('0x{0:x}'.format(JSNumber.int)) # 0x4000000000000000
```

# Enum Classes

Enum classes are intersection types and their variants normal classes.

```py
enum class PrimaryColor:
    Red(t: byte)
    Green(t: byte)
    Blue(t: byte)

    def to_byte(self):
        return self.t # Okay because the variants all have common field type
```

The example above can be desugared into the following:

```py
abstract class AbstractPrimaryColor:
    pass

@implements(AbstractPrimaryColor)
data class Red(t: byte):
    pass

@implements(AbstractPrimaryColor)
data class Green(t: byte):
    pass

@implements(AbstractPrimaryColor)
data class Blue(t: byte):
    pass

typealias PrimaryColor = Red & Green & Blue

def to_byte(variant: PrimaryColor): # This function is monomorphisable.
    return variant.t
```

The only thing we didn't capture here is the `PrimaryColor.Variant` namespace.

In the examples above, notice that the enum's variants share a single method `to_byte` that apply to all variants. We could have also used match, which through exhaustion allows monomorphisation.

```py
def to_byte(variant: PrimaryColor): # This function will be monomorphised.
    return variant.t

def to_byte(variant: PrimaryColor): # This function can also be monomorphised.
    match variant:
       case Red(t): return t
       case Blue(t): return t
       case Green(t): return t
```

Because enum variants are regular classes, we can have specialized method for them.

```py
def red_to_byte(color: PrimaryColor.Red): # This is a specialised function.
    return color.t
```

Methods and fields accessed on an intersection type must apply to all the variants.

```py
enum class Option[T]:
    Some(t: T)
    None

    def unwrap(self):
        return self.t # Error because None a variant of Option[T] does not have a `t` field.

    def unwrap(self):
        match self:
            case Some(t): return t
            case None: panic('unwrap called on None')

```

https://rust-lang.github.io/unsafe-code-guidelines/layout/enums.html

# Dynamic Dispatch

```py
givers = [StringGiver(), IntGiver()]

for giver in givers:
    giver.gift()
```

Let's assume `List` class is declared like this:

```py
class List[T]:
    # ...
```

It expects all its items to be of type `T`, but we have given it two concrete types, `StringGiver` and `IntGiver`.

When designing a statically typed language, you quickly hit type safety issues like this with `homogenous container types`. Types that store multiple items of the same memory layout usually contiguously.

A language that does not support heterogeneity is no fun. Raccoon is not such language.

`T` in `List[T]` does not necessarily mean you can only store objects of the same concrete type in list. You can also store dynamically dispatched objects. So `dyn AbstractClass` instead of `Class` or `impl AbstractClass`.

This is done transparently by the compiler.

Just like with untyped or generic functions, where the usage of the argument determines the `interface contract`, the `interface contract` of `givers` in the above example, is determined by the intersection of the collective usage of the list elements. Therefore the compiler traces all usage of `givers` elements making sure they all have a common shared implementation.

In the example above, both `StringGiver` and `IntGiver` define a `gift` method which they implement from `Giver`. So `givers` has type `List[dyn gift.1]`. Notice that we didn't say `List[dyn Givers]` since they share a parent type, `Givers`. That is because the information is not useful as all objects share a common parent `Any` (not a finalised name).

So the following is valid in Raccoon because all object share a root parent type. The type of this is `List[dyn _]` because it is not yet clear what functions the element use, which determines their `impl`.

```py
ls = [5, "Hello"]
```

The caveat however is that, operations like the one below, that you would expect to work won't compile. The compiler cannot determine at compile-time the type of an element at particular index at compile-time, so it does an exhaustive check to make sure the `__plus__` method can be used with `int` and `str` in any argument position.

```py
double = ls[0] + ls[0] # Error type of ls[0] can either be str or int and there is no __plus__(int, str) or __plus__(str, int)
```

:warning: This section is unfinished and contains a rough idea of how I want things to work.

You may wonder how the compiler determines the type of a container that stores values of different types.

```py
class Vec[T]:
    def __init__(self, capacity: int):
        self.length = 0
        self.capacity = capacity
        self.buffer = Buffer[T](capacity)

    def append(self, item: T):
        if self.length >= capacity:
            self.resize()
        self.buffer.alloc(self.length + 1, item)
        self.length += 1

    # ...

mixed = Vec() # T resolves to `[ dyn __str__.0, ... ]`
mixed.append(1) # T is int here
mixed.append("Hello") # T is str here

print(mixed[0]) # Final resolution is based on this shared method.
```

The reason this works is because Buffer implements `dyn _`.

```py
# Buffer implementation for dynamic types.
# `_` means the final type is decided by the total usage of the field.
@where(T: dyn _)
class Buffer[T]:
    def alloc(self, position: int, item: T):
        # ...

# Buffer implementation for other types with known compile-time shape.
# Concrete types instantiated from __impl__ and intersection types.
class Buffer[T]:
    def alloc(self, position: int, item: T):
        # ...
```

`Buffer[T]` gets its final type from `Vec[T]`. And since Buffer allows a `T: dyn _`, the usage of the instance method `Vec.append.T` with different types `int` and `str` made it resolve into a `dyn _`.

Any method argument that holds a value of such `T` will then be given a reference/pointer to the tagged value which will be stored on the heap. Which is the case for `append`'s `item` parameter. Raccoon does not support `dyn _` fields or variables.

Another thing worth noting is that even though the compiler resolves a `dyn _` to a dyn of field and method implementations (e.g. `[dyn __str__.0]`), the compiler does not forget (erase) the actual types in subsequent resolutions.

```py
values = [1, "Hello"] # [ dyn deep_copy.0, ... ]

# ...

def copy_first(ls):
    return ls[0].deep_copy()

# copy_first can return int or str.
# new_copy has type `int & str`.
new_copy = copy_first(values) # def copy_first(dyn _) -> int & str
```

This works because at the instantiation of `copy_first(values)`, the compiler still remembers the types behind `values: dyn _`. So it is able to check the return types of all the methods of `int & str`.

# Memory Layouting

Raccoon's memory layouting is mostly inspired by Rust's.

https://docs.google.com/presentation/d/1q-c7UAyrUlM-eZyTo1pd8SZ0qwA_wYxmPZVOQkoDmH4/edit#slide=id.p
https://github.com/pretzelhammer/rust-blog/blob/master/posts/sizedness-in-rust.md

# Type Casting

Most times the compiler won't be able to determine the type of variant or `dyn` object at compile-time. So it is useful to have type casting functions.

```py
ls = [5, "Hello"]

int_value = cast[int](ls[0])
str_value = cast[int](ls[1-1]) # Raises an error because type cannot be casted.
```

```py
variant = get_color()
red = cast[PrimaryColor.Red](variant) # Raises an error if type cannot be casted.
```

# ref vs val

By default primitive types are passed around by value and complex types by reference.

```py
age  = 55                 # primtive types are stack-livable so they are passed around by value.
name = "John"             # stack-livable part of a complex type are passed around by reference.
john = Person(name, john) # stack-livable part of a complex type are passed around by reference.
```

What is stack-livable? This is the part of a complex type that can live on the stack. Primitive types are always stack-livable.

To get a reference to a stack-allocated primitive type, use `ref`.

```py
age_ref = ref age  # age_ref points to age on the stack.
```

To get the value of a complex type's stack-livable part, use `val`.

```py
name_val = val name # name_val now contains a shallow copy of name's stack-livable part.

def who_am_i(val person):
    print(f"I am {person.name}")

who_am_i(john) # function takes a shallow copy of john's stack-livable part.
```

# Stack vs Heap Allocation

Unlike other languages, by default the _stack-livable part of a ref object_ are stored in the heap, but Raccoon optimizes for the stack, so these parts are stored on the stack where possible.

```py
age  = 45
name = "John"            # Stack-livable part allocated on the stack
john = Person(name, age) # Stack-livable part allocated on the stack
```

Racoon only stores to the heap in the following scenario: if a longer-lived object captures a reference to a shorter-lived object, the referent is going to be stored in the heap.

```py
def get_person() -> Person:
    age  = 55                # age lifetime ends in this scope
    name = "John"            # name lifetime ends in this scope so it is converted to `name = Box("James")`

    # Person has a longer lifetime.
    # name's stack-livable part is stored on the heap.
    # age is copied by value because it is a primitive type.
    return Person(name, age)

def main():
    person = get_person()
```

You can be explicit that you want a stack object to be Boxed.

```py
age   = 55
name  = Box("John")       # "John" stack-livable part is stored on the heap.
john  = Person(name, age) # Takes a copy of name which is a pointer to "James" on the heap
```

There is no way to tell the compiler you want to store an object on the stack, because it would be redundant or be a compiler error anyway.

Say we had a `stack` keyword for making things stay on the stack.

```py
def get_person() -> Person:
    age  = 55
    name = stack "James"     # name lifetime ends in this scope because we forced its stack-livable part to be allocated on the stack.
    return Person(name, age) # Person now has a dangling reference to name's stack-livable part with undefined behavior
```

This is why the compiler does not provide a way to force things to stay on the stack.

# Sync and Send

The concept of `Send` and `Sync` is borrowed from Rust.

All types are `!Sync` until they are made `Sync` by types like `Arc[Mutex[T]]`.

All types are `Send` unless they specify that they are `!Send` by implementing `!Send`, a special compiler abstract class.

When sharing ownership of objects across threads, using the `shared` keyword switches its underlying implementation from `T` or `Box[T]` to either `Arc[T]` or `Arc[Mutex[T]]`, depending on the context.

`Arc[Mutex[T]]` makes `!Sync` type `Sync` but it does not make `!Send` type `Send`.
Actually `Mutex[T]` is what makes a `!Sync` type `Sync`, the `Arc[T]` is only needed to ensure that a `Send` type is garbage-collected correctly across threads.

Raccoon does not have an `Rc[T]` type because it uses a thread-local garbage collection technique that does not require runtime reference counting.

```py
arc_total = shared 4500 # `Arc[Mutex[int]]`

handler = Thread.spawn(
    def ():
        arc_total = 4300 # captured by lambda.
)
```

Here, `Thread.spawn` instantiation contains a lambda that captures its environment, making it a closure.

`Thread.spawn` require the its lambdas to be `Send`. It's implementation looks roughly like this.

```py
class Thread:
    @where(F: Send & def() -> void)
    def spawn(fn: F):
        pass
```

# Multiple Implementations

Inspired by Rust, Raccoon allows multiple implementations of a class as long there is no conflict.

```py
@implements(Abstract[T])
class Foo[T]:
    def bar(self, value: T):
        print(f"T = {value}")

@implements(Abstract[int])
class Foo[(string, int)]:
    def bar(self, value: int):
        print(f"int = {value}")
```

# Closures

Inspired by Rust's closures.

```py
abstract class Fn[(...Args), R]:
    def call(self, args: (...Args)) -> R
```

```py
def filter(xs: [int], fn: Fn[(int), bool]) -> [int]:
    return [x for x in xs if fn(x)]

[1, 2, 3, 4, 5].filter(def (x): x % 2 == 0)
```

```py
def curry_add(x: int) -> Fn[(int), int]:
    def add(y: int) -> int:
        return x + y

    return add

fn = curry_add(5)
result = fn(10) # 15
```

For the above example, `add` is a closure that captures `x`. A temporary class is created for closures that capture variables. In this case, the compiler will generate something like this.

```py
@implements(Fn[(int), int])
class __tmp_add:
    def __init__(x: int, fn: (int) -> int):
        self.x = x
        self.fn = fn

    def call(self, args: (int)) -> int:
        (y) = args
        return self.fn(self.x, y)
```

# Futures / Streams

Inspired by Rust's futures and streams.

```py
asbstract class Future[T]:
    def poll(self, ctx: Context) -> Poll[T]

asbstract class Stream[T]:
    def poll_next(self, ctx: Context) -> Poll[Option[T]]

enum class Poll[T]:
    Ready(T),
    Pending
```

```py
@implements(Future[int])
data class Temperature(value: int = None):
    def poll(self, ctx: Context) -> Poll[int]:
        if self.value is None:
            sensor.register(def (value):
                self.value = value
                ctx.wake()
            )
            return Poll.Pending
        else:
            return Poll.Ready(self.value)
```

# Iterators

Inspired by Rust's iterators.

```py
abstract class Iterator[T]:
    def next(self) -> Option[T]
```

```py
@implements(Iterator[T])
data class ListIterator[T](xs: [T], index = 0):
    def next(self) -> Option[T]:
        if self.index >= len(self.xs):
            return None
        else:
            self.index += 1
            return self.xs[self.index - 1]

for i in ListIterator([1, 2, 3, 4, 5]):
    print(i)
```

# Exceptions vs Result Enums

Raccoon supports execeptions just because users coming from Python will have the presumption that Raccoon should have it.
Raccoon exceptions are implemented under the hood as `Result` enums. This is so that exception handling can be easier to implement and reason about.
It also makes it possible to statically infer exceptions properties in the codebase.

```py
@where(E: Error)
enum class Result[T, E]:
    Ok(T)
    Err(E)
```

As long a type implements the `Error` type, it can be used as an exception.

```py
@implements(Error)
class SomeError:
    def __init__(self, message: str):
        super().__init__(message)
```

The only major difference between Python and Raccoon exceptions is that Raccoon requires you to handle how your exceptions should be propagated.

```py
def handled() -> int:
    try:
        return get_value()
    except SomeError: # get_value() raises only SomeError, otherwise compiler will require anotating the return type with `?`.
        return 0

def unhandled() -> int?:
    value = get_value()? # Can raise here if there is an exception.
    return value
```

The `handled` function can be desugared into the following code and the compiler would still accept it.

```py
def handled() -> int:
    match get_value():
        case Ok(value): return value,
        case Err(err):
            if isinstance(err, SomeError): # This is a static check.
                return 0
            else:
                unreachable()
```

In addition to exceptions, Raccoon also supports `panic`s. A `panic` is a trap/signal that occurs when the program is in an invalid state.
Unlike Python, Raccoon panics for incidences like division by zero rather than raise a `ZeroDivisionError`.

```py
result = 5 / 0 # This does not raise a `ZeroDivisionError` exception like Python, it panics instead.
```

# Async / Await

Should have similar semantics as coroutines in the language but instead of yielding to the user, it yields to the executor.

The standard library should provide a nice default multithreaded task scheduler just like it does with heap allocator.

Reference Rust's future implementation and Tokio's scheduler implementation.

# Garbage Collection

- Automatic Reference Counting (ARC)

  Swift uses a reference counting system to determine when to deallocate a variable.

  ARC suffers from reference cycles leaks and deadlocks.

  ```py
  parent = Parent()

  """
  ParentRefs(1) = { parent }
  """

  child = Child()

  """
  ParentRefs(1) = { parent }
  ChildRefs(1) = { child }
  """

  parent.child = child

  """
  ParentRefs(1) = { parent }
  ChildRefs(2) = { child, parent.child }
  """

  child.parent = parent

  """
  ParentRefs(2) = { parent, child.parent }
  ChildRefs(2) = { child, parent.child }
  """

  """
  DEALLOCATION POINT
  """

  """
  `parent` goes out of scope
  -> `parent` decrements rc.
  -> Parent object does not deinitialise because its rc is not 0 yet.

  ParentRefs(1) = { child.parent }
  ChildRefs(2) = { child, parent.child }
  """

  """
  `child` goes out of scope
  -> `child` decrements rc.
  -> Child object does not deinitialise because its rc is not 0 yet.

  ParentRefs(1) = { child.parent }
  ChildRefs(1) = { parent.child }
  """
  ```

  We can introduce a weak reference to break the cycle.

  ```py
  child.parent = WeakRef(parent)

  """
  ParentRefs(1) = { parent }
  ChildRefs(2) = { child, parent.child }
  """

  """
  DEALLOCATION POINT
  """

  """
  name `parent` goes out of scope
  -> `parent` decrements rc.
  -> Parent object rc is 0 so it deinitialises.
  -> `parent.child` decrements Child rc.

  ParentRefs(0) = { }
  ChildRefs(1) = { child }
  """

  """
  `child` goes out of scope
  -> `child` decrements rc because no one refers to it anymore.
  -> Child object rc is 0 so it deinitialises.
  -> `child.parent` is already nil.

  ParentRefs(0) = { }
  ChildRefs(0) = { }
  """
  ```

  The reason there is a reference cycle problem at all is that object's in Swift's world can't decrement internal references until the object's rc is zero which then triggers deinitialization.

  https://developer.apple.com/videos/play/wwdc2021/10216/

  https://docs.swift.org/swift-book/LanguageGuide/AutomaticReferenceCounting.html

- Static Reference Tracking (SRT) [WIP]

  Enter SRT. I'm proposing a different style of ARC that is not susceptible to reference cycles. I'm going to call it `Static Reference Tracking` for now because I am not aware of any literature on it.

  Static Reference Tracking (SRT) is a deallocation technique that tracks objects' lifetimes at compile-time and can break reference cycles statically. It also does not have runtime reference counting operations like ARC does.

  ```
  foo () {
      a      = Obj1()
      b      = Obj2()
      c      = Obj3()
      d      = Obj4()

      # We call `free_owned_deallocatable` after every last use of an arg passed by value or a local-lifetime variable.
      free_owned_deallocatable :: b

      c <- a = Obj3() <- Obj1()

      # Foo has objects it needs inner function frames to deallocate, so it sets pointer to deallocation list.
      set_global_deallocatable_ptr

      bar (ref c, ref a, ref d) { # bar frame; knows nothing about caller function frame
          a <- c = Obj1() <- Obj3() # Cross-frame cycle detection!
          e      = Obj5()

          free_owned_deallocatable :: e

          # We call `free_transferred_deallocatable` after every last use of an arg passed by ref.
          free_transferred_deallocatable :: a, c

          qux (ref d) { # qux frame; knows nothing about caller function frame
              free_transferred_deallocatable :: d
          }
      }
  }
  ```

  `free_transferred_deallocatable` deallocates what it needs to and increments the ptr.

  #### DEALLOCATABLE LIST

  Each thread has its own thread-local `DEALLOCATABLE_LIST` since this model doesn't work with objects across between threads. You need `Arc[T]` for that.

  The compiler can arrange the `DEALLOCATABLE_LIST` during compilation because a caller function frame always knows the structure of its inner function frames.

  ```
  DEALLOCATABLE_PTR -> points to -> DEALLOCATABLE_LIST

  DEALLOCATABLE_LIST:
      foo:
          (stack_livable_ptr_address: ptr _), :: a
          (stack_livable_ptr_address: ptr _), :: c
          (stack_livable_ptr_address: ptr _), :: d
      ...
  ```

  In this case, the compiler lays out how the inner function frames of `foo` should deallocate `foo`'s transferred objects. `stack_livable_ptr_address` is the address of the stack-livable pointer pointing to the heap. We don't actually hold heap addresses because of invalidation that can take place.

  #### HOW IT PREVENTS REFERENCE CYCLES

  The compiler tracks every object in the program just like ARC, but unlike ARC tracks all the objects a **name** is refers to. This includes internal references (i.e. fields) of the name. The compiler decrements all the associated object rc when the **name** lifetime ends. The counting is done at compile-time so there is no runtime aspect to it.

  #### POINTER ALIASING

  Raw pointer aliasing affects all dellocation techniques. SRT, Tracing GCs, ARC, ownership semantics, etc. That is why we have references. They are an abstraction over pointers, something our GCs understand. Raw pointer misuse is a problem for any GC technique.

  #### REFERENCE INTO A LIST

  If there is a reference to a list item, the entire list is not freed until all references to it and/or its elements are dead.

  ```py
  scores = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

  fourth = scores[3]

  some = scores[3:7]
  ```

  #### SELF-REFERENCING OBJECT ISSUE

  TODO(appcypher): How does Raccoon handle self-referential structs getting passed around by value in memory? A potential solution is to detect them and "Pin" so they are not possible to pass by value.

##### REFERENCES

https://stackoverflow.com/questions/48986455/swift-class-de-initialized-at-end-of-scope-instead-of-after-last-usage

https://forums.swift.org/t/should-swift-apply-statement-scope-for-arc/4081
