<div align="center">
    <a href="#" target="_blank">
        <img src="https://raw.githubusercontent.com/raccoon-lang/raccoon/master/raccoon.svg" alt="Raccoon Logo" width="140" height="140"></img>
    </a>
</div>

<h1 align="center">Raccoon</h1>

`raccoon` is a statically-typed language that borrows syntax from Python and adopts some Rust philosophies resulting in a language that is both easy to learn and powerful to use.

`raccoon` encourages the rapid prototyping spirit of Python with little to no compromise on the performance and safety that Rust provides.

Below is an example of what Raccoon currently looks like:

```py
class Person:
    """
    Class for creating details about a person.
    """

    population = 0

    def init(self, name, age):
        """
        Creates a new person
        """

        self.name = name
        self.age = age

        Person.population += 1

    def del(self):
        """
        Decrement population
        """

        Person.population -= 1

    def debug(self, f):
        """
        Create a string representation of object
        """

        return f.debug_class("Person")
            .field("name", self.name)
            .field("age", self.age)

jane = Person("Jane Doe", 23)
print("jane =", jane)
```
