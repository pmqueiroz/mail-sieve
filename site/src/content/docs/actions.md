---
title: Bulk actions
description: Archive, trash, mark, flag, move or permanently delete thousands of messages, and undo moves.
lead: Every action shows how many messages it will touch before it runs.
group: Using mailsieve
order: 3
---

| Action | What happens |
| --- | --- |
| Archive | Moves to your archive folder. On Gmail it removes the Inbox label |
| Trash | Moves to Trash. Your provider empties it on its own schedule |
| Mark read / unread | Sets or clears the `\Seen` flag |
| Flag / unflag | Sets or clears the `\Flagged` flag |
| Move to folder | Any existing IMAP folder |
| Delete permanently | Removes messages from the server. Asks for confirmation and cannot be undone |

## Unsubscribe

Unsubscribe uses the sender's `List-Unsubscribe` header: an RFC 8058 one-click request when offered, otherwise a mailto through your account, otherwise the link opens for you to finish. You can also archive or trash everything that sender sent in the same step.

## Undo

Finished actions can be undone from the job toast or from Activity, once. Moves are found again by Message-ID and moved back.

:::warning
Permanent delete is never undoable, and messages without a Message-ID cannot be restored by undo.
:::
