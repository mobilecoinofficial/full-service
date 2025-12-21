from mobilecoin import util

def test_receipt_base64_roundtrip():
    receipt_base64 = "CiIKIDh3CQjga4l3nsdjYqpLjDTu7q1FpHw8yuswMYAvIyM+EiIKIAS+8HNQYi87z0bSSxPTPmbSyzgiVkeiM6qNneFiOzBlGOqhzAEqNwoiCiD4u4nulPc+MYVZhezmVZuzxpsDtZcW5pFAF0C/RV6kIhGgHmCg7NTypxoIC/Jffzxfy2c="
    receipt_json = util.b64_receipt_to_full_service_receipt(receipt_base64)
    receipt_base64_again = util.full_service_receipt_to_b64_receipt(receipt_json)
    assert receipt_base64 == receipt_base64_again
