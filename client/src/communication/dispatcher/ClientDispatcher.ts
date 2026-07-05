import { createCommunicationId } from "../shared/id"
import type {
  Command,
  Event as CommunicationEvent,
  Message,
} from "../types"
import { ClientEventManager } from "../event/ClientEventManager"
import { ClientRequestManager } from "../request/ClientRequestManager"
import { ResponseDispatcher } from "./ResponseDispatcher"

export type CommandHandler<TBody = unknown> = (
  message: Message<TBody>,
) => void | Promise<void>

export class ClientDispatcher {
  private readonly commandHandlers = new Map<Command, CommandHandler>()
  private readonly responses: ResponseDispatcher

  constructor(
    requests: ClientRequestManager,
    private readonly events: ClientEventManager,
  ) {
    this.responses = new ResponseDispatcher(requests)
  }

  registerHandler(command: Command, handler: CommandHandler) {
    this.commandHandlers.set(command, handler)
  }

  removeHandler(command: Command) {
    return this.commandHandlers.delete(command)
  }

  async dispatch(message: Message) {
    switch (message.header.messageType) {
      case "response":
        this.responses.dispatch(message)
        return
      case "event":
      case "broadcast":
      case "notification":
        await this.dispatchEvent(message)
        return
      case "request":
        await this.dispatchCommand(message)
        return
      default:
        return
    }
  }

  private async dispatchEvent<TBody>(message: Message<TBody>) {
    const event: CommunicationEvent<TBody> = {
      id: createCommunicationId("evt"),
      name: message.header.command,
      payload: message.body,
      source: "communication",
      priority: 0,
      timestamp: Date.now(),
    }

    await this.events.publish(event)
  }

  private async dispatchCommand(message: Message) {
    const handler = this.commandHandlers.get(message.header.command)

    if (!handler) {
      return
    }

    await handler(message)
  }
}
