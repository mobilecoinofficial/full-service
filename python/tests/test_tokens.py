from mobilecoin.tokens import Token


def test_format():
    mob = Token(0, 'MobileCoin', 'MOB', 12, 4)

    def f(x):
        value = mob.deconvert(x)
        return str(mob.format(value, extra_precision=True))

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
