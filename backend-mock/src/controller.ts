import { Application, Request, Response } from "express";

export class MainController {
    public registerRoutes(app: Application) {
        app.get("/healthcheck", this.healthCheck.bind(this));
        app.get("/allocation/:chain/:address/:denom", this.allocation.bind(this));
        app.post("/claim/:chain/:address/:denom", this.claim.bind(this));
    }

    private healthCheck(req: Request, res: Response) {
        res.status(200);
        res.send("OK");
    }

    private allocation(req: Request, res: Response) {
        const { params } = req;
        setTimeout(() => {
            return res.send({
                "allocation": "1293819221",
                "denom" : params.denom,
                "has_claimed": false,
                "chain": params.chain,
                "address": params.address
            });
        }, 1000)
    }

    private claim(req: Request, res: Response) {
        setTimeout(() => {
            return res.send(
                {
                    "has_claimed": false,
                    "transaction_hash": "3A41C57DDBFA6F68052947BC3204CE9DB64767CA907E899D9EDF9DB88E0896C5"
                });
        }, 1000)

    }
}
