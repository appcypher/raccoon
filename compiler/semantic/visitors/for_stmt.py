"""
"""

from compiler.ast import (
    Integer,
)
from compiler import Visitor


class ForVisitor(Visitor):
    """
    """

    def __init__(self, ast, info):
        self.return_stmt = ast
        self.info = info

    def start_visit(self):
        self.return_stmt.accept(self)

    def act(self):
        """
        """

        return False
