import pytest
from mobilecoin.token import get_token, Amount


MOB = get_token('MOB')
EUSD = get_token('EUSD')


def test_format():

    def f(x):
        amount = Amount.from_display_units(x, MOB)
        return str(amount.format())

    assert f('4200') == '4200.0000 MOB'
    assert f('42') == '42.0000 MOB'
    assert f('42.0') == '42.0000 MOB'
    assert f('42.0000') == '42.0000 MOB'
    assert f('4.2') == '4.2000 MOB'
    assert f('4.20') == '4.2000 MOB'
    assert f('.42') == '0.4200 MOB'
    assert f('.0042') == '0.0042 MOB'
    assert f('.004200') == '0.0042 MOB'
    assert f('.00000042') == '0.00000042 MOB'


def test_get_token():
    assert get_token(0).short_code == 'MOB'
    assert get_token('MOB').short_code == 'MOB'
    assert get_token('mob').short_code == 'MOB'
    assert get_token('Mob').short_code == 'MOB'

    assert get_token(1).short_code == 'eUSD'
    assert get_token('EUSD').short_code == 'eUSD'
    assert get_token('eUSD').short_code == 'eUSD'
    assert get_token('eusd').short_code == 'eUSD'


def test_amount_comparison():
    assert (
        Amount.from_display_units(3, MOB) ==
        Amount.from_display_units(3, MOB)
    )
    assert (
        Amount.from_display_units(3, MOB) <
        Amount.from_display_units(4, MOB)
    )
    assert (
        Amount.from_display_units(3, MOB) <=
        Amount.from_display_units(4, MOB)
    )
    assert (
        Amount.from_display_units(3, MOB) <=
        Amount.from_display_units(3, MOB)
    )
    assert (
        Amount.from_display_units(4, MOB) >
        Amount.from_display_units(3, MOB)
    )
    assert (
        Amount.from_display_units(4, MOB) >=
        Amount.from_display_units(3, MOB)
    )
    assert (
        Amount.from_display_units(3, MOB) >=
        Amount.from_display_units(3, MOB)
    )
    with pytest.raises(AssertionError):
        Amount.from_display_units(3, MOB).__eq__(3)
    with pytest.raises(AssertionError):
        Amount.from_display_units(3, MOB).__eq__('3')
    with pytest.raises(AssertionError):
        Amount.from_display_units(3, MOB).__eq__(Amount.from_display_units(3, EUSD))


def test_token_hash():
    d = {MOB: 3, EUSD: 4}
    assert d[MOB] == 3
    assert d[EUSD] == 4
