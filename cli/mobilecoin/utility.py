from decimal import Decimal


PMOB = Decimal('1e12')


def mob2pmob(x):
    """ Convert from MOB to picoMOB. """
    return round(Decimal(x) * PMOB)


def pmob2mob(x):
    """ Convert from picoMOB to MOB. """
    result = int(x) / PMOB
    if result == 0:
        return Decimal('0')
    else:
        return result


def try_int(x):
    if x is not None:
        return int(x)
