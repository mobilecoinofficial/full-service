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

    def convert(self, value):
        """
        Convert fixed-point stored units of this token into displayed units. 
        """
        value = Decimal(int(value))
        result = value / self.conversion_factor()
        if result == 0:
            result = Decimal("0")
        return result

    def deconvert(self, display_value):
        """
        Convert displayed units of this token into fixed-point stored units.
        """
        display_value = Decimal(display_value)
        result = round(display_value * self.conversion_factor())
        return result

    def format(self, value, extra_precision=False):
        display_value = self.convert(value)
        precision = self.suggested_precision

        if extra_precision:
            normalized = display_value.normalize()
            _, _, exponent = normalized.as_tuple()
            precision = max(-exponent, precision)

        return '{:.{}f} {}'.format(
            display_value,
            precision,
            self.short_code,
        )



TOKENS = [
    Token(0, 'MobileCoin', 'MOB', 12, 4),
    Token(1, 'Electronic Dollar', 'eUSD', 6, 2),
]
TOKENS_BY_ID = { t.token_id: t for t in TOKENS }

def get_token(token_id):
    return TOKENS_BY_ID[int(token_id)]
