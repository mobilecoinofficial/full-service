# Getting Started

As a member of an Exchange, who will be writing the Wallet Interface code and setting up the integration between the
Exchange and the MobileCoin Network, you will need to understand the four main steps of the MobileCoin Network and
Exchange integration.

This guide is specifically for Exchange Interface Engineers (EIEs) who have chosen to enable customers to transact
cryptocurrency using MOB and its payment system, Full Service. For more information about the MobileCoin Network, check
out
the [MobileCoin Ecosystem](https://docs.google.com/document/d/1hDU2hHjnMdqQtfkCPxcq__BbFNkDN1iDCm6Jw1kJD3g/edit?usp=sharing) -
a brief document about the MobileCoin open-source software network.

In setting up the integration between the MobileCoin Network and the Exchange, the EIE will need to follow four major
steps:

1. Using the Exchange’s own integration software to exercise the endpoints, or optionally using the Python library
   provided by MobileCoin, the EIE will write the Exchange’s Wallet Interface to be able to integrate it with the
   MobileCoin Network through Full Service’s endpoints.
2. Run Full Service.&#x20;
3. Register the Exchange’s accounts with the Wallet API.&#x20;
4. Make withdrawals and deposits.

{% hint style="info" %}
[Subaddresses](../../glossary/subaddress.md) are introduced and may be a new concept to Exchanges. But, it is a very
important detail that enables KYC for Exchanges and must be set up in the Wallet Interface before transactions can
occur. The importance of subaddresses will be explained when setting up public addresses and request codes, which enable
customers to send MOB.
{% endhint %}

<figure><img src="../../.gitbook/assets/4 steps.png" alt=""><figcaption><p>An overview of how the Full Service payment system integrates with the Exchange</p></figcaption></figure>
