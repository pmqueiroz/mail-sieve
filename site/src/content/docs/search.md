---
title: Search syntax
description: Every mail-sieve search filter, from sender and age to size, state and category.
lead: Filters combine with AND. Prefix any filter with a minus to negate it.
group: Using mail-sieve
order: 2
---

| Filter | Meaning |
| --- | --- |
| `from:deals@shop.com` | Exact sender |
| `from:*@shop.com` | Any address at a domain |
| `from:alice` | Sender address or name contains |
| `domain:shop.com` | Sender domain, including subdomains |
| `subject:invoice` | Subject contains |
| `list:newsletter` | Mailing list id contains |
| `in:inbox` | Folder, or Gmail label |
| `is:unread` `is:read` `is:flagged` `is:answered` | State |
| `has:unsubscribe` `has:list` | Newsletter signals |
| `older_than:1y` `newer_than:7d` | Age, units `h d w m y` |
| `before:2024-01-01` `after:2023-06-01` | Dates |
| `larger:5mb` `smaller:100kb` | Size |
| `category:newsletters` | One of `newsletters notifications promotions receipts social people` |
| `-from:boss@work.com` | Exclude a match |
| `receipt` | Free text over subject and sender |

## Example

```text
has:list older_than:1y is:unread -from:*@github.com
```
