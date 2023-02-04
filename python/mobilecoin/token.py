from dataclasses import dataclass
from decimal import Decimal

@dataclass
class Token:
    token_id: int
    currency_name: str
    short_code: str
    decimals: int
    suggested_precision: int

    def conversion_factor(self):
        return Decimal(10**self.decimals)

    def convert(self, value: int) -> Decimal:
        """
        Convert fixed-point storage units of this token into Decimal displayed units. 
        """
        result = Decimal(int(value)) / self.conversion_factor()
        if result == 0:
            result = Decimal("0")
        return result

    def deconvert(self, display_value) -> int:
        """
        Convert Decimal displayed units of this token into fixed-point storage units.
        """
        display_value = Decimal(display_value)
        return int(round(display_value * self.conversion_factor()))


TOKENS = [
    Token(0, 'MobileCoin', 'MOB', 12, 4),
    Token(1, 'Electronic Dollar', 'eUSD', 6, 2),
]
TOKENS_BY_ID = { t.token_id: t for t in TOKENS }
TOKENS_BY_SHORT_CODE = { t.short_code.lower(): t for t in TOKENS }


def get_token(token):
    """
    Look up a token by its integer ID number, or by its short code string.
    """
    if isinstance(token, Token):
        return token

    try:
        token_id = int(token)
        return TOKENS_BY_ID[int(token_id)]
    except ValueError:
        short_code = str(token).lower()
        return TOKENS_BY_SHORT_CODE[short_code]


class Amount:
    def __init__(self, value: int, token, _inner_constructor=False):
        if not _inner_constructor:
            raise ValueError("Use Amount.from_display_units or Amount.from_storage_units")
        self.value = value
        self.token = token

    @staticmethod
    def from_display_units(display_value, token):
        token = get_token(token)
        return Amount(
            token.deconvert(display_value),
            token,
            _inner_constructor=True,
        )

    @staticmethod
    def from_storage_units(value, token):
        assert not isinstance(value, Amount)
        return Amount(
            int(value),
            get_token(token),
            _inner_constructor=True,
        )

    def __add__(self, other):
        assert isinstance(other, Amount)
        assert self.token == other.token
        return Amount.from_storage_units(self.value + other.value, self.token)

    def __sub__(self, other):
        assert isinstance(other, Amount)
        assert self.token == other.token
        return Amount.from_storage_units(self.value - other.value, self.token)

    def __lt__(self, other):
        assert isinstance(other, Amount)
        assert self.token == other.token
        return self.value < other.value

    def __eq__(self, other):
        assert isinstance(other, Amount)
        assert self.token == other.token
        return self.value == other.value

    def display_value(self):
        return self.token.convert(self.value)

    def format(self, extra_precision=False):
        """
        Takes a display value representing an amount of a token and formats it
        with correct precision and short code suffix.
        """
        display_value = self.display_value()
        precision = self.token.suggested_precision

        if extra_precision:
            normalized = display_value.normalize()
            _, _, exponent = normalized.as_tuple()
            precision = max(-exponent, precision)

        return '{:.{}f} {}'.format(
            display_value,
            precision,
            self.token.short_code,
        )


