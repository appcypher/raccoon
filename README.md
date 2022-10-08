<div align="center">
    <a href="#" target="_blank">
        <img src="https://raw.githubusercontent.com/raccoon-lang/raccoon/master/raccoon.svg" alt="Raccoon Logo" width="140" height="140"></img>
    </a>
</div>

<h1 align="center">Raccoon</h1>

`raccoon` is a language with Python 3.x syntax that is amenable to static analysis. The repository both defines the spec of the language and contains a reference implementation of the compiler.

**Raccoon will not maintain full syntactic and semantic compatibility with Python**. Several dynamic elements known of Python are not available in Raccoon. While Raccoon prioritizes a design that benefits static analysis, it still allows Python's level of flexibility where statically determinable.

Raccoon compiler implementation generates WebAssembly code.

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
