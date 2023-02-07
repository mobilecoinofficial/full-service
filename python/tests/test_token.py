from mobilecoin.token import Token, get_token, Amount


def test_format():
    mob = Token(0, 'MobileCoin', 'MOB', 12, 4)

    def f(x):
        amount = Amount.from_display_units(x, mob)
        return str(amount.format(extra_precision=True))

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

    assert get_token(1).short_code == 'eUSD'
    assert get_token('eUSD').short_code == 'eUSD'
    assert get_token('eUsd').short_code == 'eUSD'
    assert get_token('eusd').short_code == 'eUSD'
