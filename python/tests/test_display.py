from mobilecoin.cli import _format_decimal


def test_format_decimal():
    def f(x):
        return str(_format_decimal(x))
    assert f('4200') == '4200'
    assert f('42') == '42'
    assert f('42.0') == '42'
    assert f('42.0000') == '42'
    assert f('4.2') == '4.2'
    assert f('4.20') == '4.2'
    assert f('.42') == '0.42'
    assert f('.0042') == '0.0042'
    assert f('.004200') == '0.0042'
    assert f('.00000042') == '0.00000042'
