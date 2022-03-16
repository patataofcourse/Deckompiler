import click

@click.group(name="deckompiler")
def cli():
    pass

class Int(click.ParamType):
    name = "integer"

    def convert(self, value, param, ctx):
        if isinstance(value, int):
            return value

        try:
            if value[:2].lower() == "0x":
                return int(value[2:], 16)
            elif value[:2] == "0o":
                return int(value[2:], 8)
            elif value[:2] == "0b":
                return int(value[2:], 2)
            return int(value, 10)
        except ValueError:
            self.fail(f"{value!r} is not a valid integer", param, ctx)