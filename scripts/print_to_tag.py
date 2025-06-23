import sys
from typing import Iterator


def main():
    for comment, variable in to_2tuple(sys.stdin):
        tag = comment.split()[1]
        variable = variable[:-2]
        print(f'Self::{variable} => "{tag}",')


def to_2tuple(lines: Iterator[str]) -> Iterator[tuple[str, str]]:
    a = next(lines, None)
    b = next(lines, None)
    if a and b:
        yield a, b
        yield from to_2tuple(lines)


if __name__ == '__main__':
    main()

