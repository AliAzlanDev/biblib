import sys
from typing import Iterator

def main():
    lines = map(str.strip, sys.stdin)
    for a, b, c in to_three_tuples(lines):
        print(f'/// {a} - {b}: {c}')
        print(to_variable_name(b), end=',\n')


def to_three_tuples(lines: Iterator[str]) -> Iterator[tuple[str, str, str]]:
    a = next(lines, None)
    b = next(lines, None)
    c = next(lines, None)
    if a and b and c:
        yield a, b, c
        yield from to_three_tuples(lines)


def to_variable_name(s: str) -> str:
    words = s.title().split()
    return sanitize_chars(''.join(words))


def sanitize_chars(s: str) -> str:
    return ''.join(x for x in s if x.isalnum())


if __name__ == '__main__':
    main()

