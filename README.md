<div align="center">
    <a href="#" target="_blank">
        <img src="https://raw.githubusercontent.com/raccoon-lang/raccoon/master/raccoon.svg" alt="Raccoon Logo" width="140" height="140"></img>
    </a>
</div>

<h1 align="center">Raccoon</h1>

`raccoon` is a statically-typed language that borrows syntax from Python but adopts some Rust philosophies resulting in a language that is both easy to learn and powerful to use.

`raccoon` encourages the rapid prototyping spirit of Python with little to no compromise on the performance and safety that Rust provides.

Below is an example of what Raccoon currently looks like:

```py
class Person:
    """
    Class for creating a person.
    """

    def init(self, name, age):
        """
        Creates a new person
        """
        self.name = name
        self.age = age

    def debug(self, fmt):
        """
        Create a string representation of object
        """
        fmt.debug_class("Person")
            .field("name", self.name)
            .field("age", self.age)
            .finish()

jane = Person("Jane Doe", 23)

print(f"jane = {jane}")
```
