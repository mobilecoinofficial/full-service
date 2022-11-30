class Account(object):
    def __init__(self, d):
        self.__dict__ = d

class Response(object):
    def __init__(self, d):
        self.__dict__ = d
        if self.method == 'get_accounts':
            self.account_ids = self.result['account_ids']
            self.accounts = {}
            for acc in self.account_ids:
                self.accounts[acc] = Account(self.result['account_map'][acc])
        if self.method == 'get_account_status':
            self.account = Account(self.result['account'])
