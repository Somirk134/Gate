import { useAppContext } from "@/providers/appContext"
import type { ServiceToken } from "@/registry/ServiceRegistry"

export function useService<T>(token: ServiceToken<T>): T {
  return useAppContext().services.resolve(token)
}
