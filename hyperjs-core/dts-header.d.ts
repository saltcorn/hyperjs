export type JsHandlerFn = (req: Request, res: Response) => boolean | void | Promise<boolean | void>
export type JsVerifyFn = (req: Request, res: Response, buf: Buffer, encoding: String) => void
export type JsSetHeadersFn = (res: Response, path: string, stat: any) => void
