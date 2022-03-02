<div align="center">
    <a href="#" target="_blank">
        <img src="https://raw.githubusercontent.com/raccoon-lang/raccoon/master/raccoon.svg" alt="Raccoon Logo" width="140" height="140"></img>
    </a>
</div>

<h1 align="center">Raccoon</h1>

`raccoon` is a language with Python 3.x syntax that is amenable to static analysis. The repository both defines the spec of the language and contains a reference implementation of the compiler.

**Raccoon will not maintain full syntactic and semantic compatibility with Python**. Several dynamic elements known of Python are not available in Raccoon. While Raccoon prioritizes a design that benefits static analysis, it still allows Python's level of flexibility where statically determinable.

Raccoon compiler implementation generates WebAssembly code.

Below is an example of what Raccoon looks like:

```py
class Person:
    """
    Class for creating details about a person.
    """

    population = 0

    def __init__(self, name, age, gender="Male"):
        self.name = name
        self.age = age
        self.gender = gender
        Person.population += 1

    def __del__(self):
        """
        Decrement population
        """
        Person.population -= 1

    def __str__(self):
        """
        Create a string representation of object
        """
        return f"Person(name={self.name}, age={self.age}, gender={self.gender})"


jane = Person("Jane Doe", "Female", 23)
print("Jane >", jane)
```
