#!/usr/bin/env python3
import http.server, socketserver, json, os, urllib.parse, uuid, time
from pathlib import Path

ROOT = Path(__file__).parent.parent / "client-godot"
PORT = 8001

# In-memory mock DB
users = {}
tokens = {}

class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=str(ROOT), **kwargs)

    def end_headers(self):
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Authorization, Content-Type")
        super().end_headers()

    def do_OPTIONS(self):
        self.send_response(200)
        self.end_headers()

    def do_GET(self):
        parsed = urllib.parse.urlparse(self.path)
        path = parsed.path
        query = urllib.parse.parse_qs(parsed.query)
        print(f"GET {path}")

        # Raiz deve servir o jogo, não listagem de arquivos
        if path == "/" or path == "/index.html":
            self.path = "/export/web/index.html"

        if path == "/health":
            self.send_json({"status":"ok","database":"ok","redis":"ok","mock":True})
            return
        if path.startswith("/api/v1/"):
            # Mock API responses
            if path.startswith("/api/v1/characters"):
                self.send_json([{"id":str(uuid.uuid4()),"name":"Hero","gender":"male","class":"commander","subclass":"emperor","level":5,"power_rating":1250,"is_leader":True}])
                return
            if path.startswith("/api/v1/squad"):
                self.send_json([{"slot":1,"name":"Hero","level":5,"class":"commander","subclass":"emperor"}])
                return
            if path.startswith("/api/v1/inventory"):
                self.send_json({"items":[{"id":str(uuid.uuid4()),"name":"Espada da Floresta","rarity":"common","enhancement":2}],"offset":0,"limit":50})
                return
            if path.startswith("/api/v1/cosmetics"):
                self.send_json([{"cosmetic_type":"wings","tier":2,"stars":3,"fragments":12,"essences":2},{"cosmetic_type":"mount","tier":1,"stars":5,"fragments":8,"essences":1}])
                return
            if path.startswith("/api/v1/chat/global"):
                self.send_json([{"sender_name":"System","content":"Bem-vindo à Ascendia! Mock API ativa."}])
                return
            if path.startswith("/api/v1/rankings/power"):
                self.send_json({"entries":[{"rank":1,"display_name":"HeroMock","power_rating":4200,"level":25,"character_name":"Hero"}],"offset":0,"limit":20,"rebuilt":False})
                return
            if path.startswith("/api/v1/vip/status"):
                self.send_json({"vip_level":3,"vip_points":450,"next_level_points":600,"benefits":["+15% EXP","+10% Drop"]})
                return
            if path.startswith("/api/v1/battle-pass"):
                self.send_json({"season":{"name":"Season 1 — Inferno","starts_at":"2026-08-18","ends_at":"2026-09-17"},"progress":{"level":5,"xp":5200,"premium":False},"next_level_xp":6000})
                return
            if path.startswith("/api/v1/tower/status"):
                self.send_json({"current_floor":12,"best_floor":15,"next_floor":16,"is_boss":False,"rewards_preview":"Fragmentos + Gold"})
                return
            if path.startswith("/api/v1/characters") and "/stats" in path:
                self.send_json({"hp":1850,"attack":245,"defense":180,"attack_speed":1.35,"crit_rate":0.12,"crit_damage":1.65,"luck":0.08,"accuracy":0.05,"dodge":0.07,"penetration":0.08,"power_rating":2850})
                return
            # Fallback
            self.send_json({"mock":True,"path":path})
            return

        # Static files
        return super().do_GET()

    def do_POST(self):
        parsed = urllib.parse.urlparse(self.path)
        path = parsed.path
        length = int(self.headers.get('Content-Length', 0))
        body = self.rfile.read(length).decode() if length else "{}"
        try:
            data = json.loads(body) if body else {}
        except:
            data = {}
        print(f"POST {path} {data}")

        # Auth mock - always succeed
        if path in ["/api/v1/auth/register","/api/v1/auth/login","/api/v1/auth/oauth/google","/api/v1/auth/oauth/discord"]:
            uid = str(uuid.uuid4())
            access = f"mock_access_{uid[:8]}"
            refresh = f"mock_refresh_{uid[:8]}"
            self.send_json({"access_token":access,"refresh_token":refresh,"user_id":uid,"token_type":"Bearer","expires_in":900})
            return
        if path == "/api/v1/auth/refresh":
            self.send_json({"access_token":"mock_access_new","refresh_token":"mock_refresh_new","user_id":str(uuid.uuid4())})
            return
        if path == "/api/v1/combat/start":
            stage = data.get("stage",1)
            self.send_json({
                "combat_id": str(uuid.uuid4()),
                "stage": stage,
                "victory": True,
                "duration_ms": 4200,
                "gold": 45,
                "experience": 32,
                "seed": 123456789,
                "drop_rarity": "rare",
                "level_up": None,
                "stars": 3,
                "events":[
                    {"wave":1,"enemy":"slime","enemy_count":3,"cleared":True,"sequence":1},
                    {"wave":2,"enemy":"goblin","enemy_count":2,"cleared":True,"sequence":2},
                    {"wave":3,"enemy":"troll","enemy_count":1,"cleared":True,"sequence":3}
                ]
            })
            return
        if path.startswith("/api/v1/inventory/equip") or path.startswith("/api/v1/inventory/enhance") or path.startswith("/api/v1/cosmetics/upgrade"):
            self.send_json({"success":True,"enhancement":3,"fragments_spent":15,"tier":2,"stars":4,"tier_up":False})
            return
        if path.startswith("/api/v1/chat/global"):
            self.send_json({"id":str(uuid.uuid4()),"sender_name":"Você","content":data.get("content",""),"created_at":time.time()})
            return
        if path.startswith("/api/v1/offline-rewards/claim"):
            self.send_json({"gold":120,"experience":45,"elapsed_seconds":3600,"replayed":False})
            return
        if path.startswith("/api/v1/vip/grant"):
            self.send_json({"vip_level":4,"vip_points":950,"granted":500})
            return
        if path.startswith("/api/v1/tower/challenge"):
            self.send_json({"floor":13,"victory":True,"current_floor":13,"best_floor":15,"gold":85,"xp":42,"events":[]})
            return
        if path.startswith("/api/v1/arena/fight"):
            self.send_json({"victory":True,"my_power":2850,"opp_power":2600,"new_rating":1150,"tier":"prata"})
            return
        if path.startswith("/api/v1/dungeons/run"):
            self.send_json({"dungeon_type":data.get("type","exp"),"gold":50,"xp":80,"frags":4})
            return
        if path.startswith("/api/v1/enchant"):
            self.send_json({"enchanted":data.get("inventory_item_id"),"rolled_stats":{"crit_rate":0.03}})
            return
        # Generic success
        self.send_json({"ok":True,"mock":True,"path":path})

    def send_json(self, obj, status=200):
        body = json.dumps(obj).encode()
        self.send_response(status)
        self.send_header("Content-Type","application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

os.chdir(ROOT)
with socketserver.TCPServer(("0.0.0.0", PORT), Handler) as httpd:
    print(f"Mock Ascendia API + Godot static em 0.0.0.0:{PORT} — /health, /api/v1/*, /*")
    httpd.serve_forever()
