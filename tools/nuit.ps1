# ===========================================================================
#  fabbot : travaux de nuit sur la Surface (chantier 0 de la feuille de route)
#
#  Lance, l'un apres l'autre, les travaux de la liste (tournois, mesures...),
#  et ecrit les resultats dans resultats\nuit\AAAA-MM-JJ\ :
#    resume.md        : le resume a lire le matin (une section par travail)
#    NN-nom.txt       : la sortie complete de chaque travail
#    travaux.txt      : la liste qui a ete lancee
#
#  Liste des travaux, par ordre de priorite :
#    1. resultats\nuit\a-faire.txt s'il existe (liste du soir, deposee par
#       une session Claude ou a la main ; deplacee dans le dossier du jour) ;
#    2. sinon tools\nuit\travaux.txt (liste par defaut, suivie par git).
#  Format d'une ligne :   nom | programme | arguments
#  (programme = un programme du paquet fab-sim : fab-sim, analyse...)
#
#  Options :
#    -Essai         : liste courte tools\nuit\essai.txt (quelques minutes),
#                     pour verifier que tout marche ;
#    -Fin 07:30     : heure limite ; un travail encore en cours est arrete,
#                     les suivants ne sont pas lances ;
#    -Discret       : priorite basse (pour garder le PC utilisable le jour) ;
#    -SansMiseAJour : ne fait pas le « git pull » ;
#    -Planifie      : lancement par la tache planifiee (23 h, deverrouillage
#                     ou reveil du PC) : le script ne fait rien hors de la
#                     plage -Heure .. -Fin (moins 1 h), ni si la nuit a deja
#                     ete lancee ;
#    -Heure 23:00   : debut de la plage (avec -Planifie).
#  Mode d'emploi : docs\nuit.md
# ===========================================================================
param(
    [switch]$Essai,
    [string]$Fin = "07:30",
    [switch]$Discret,
    [switch]$SansMiseAJour,
    [switch]$Planifie,
    [string]$Heure = "23:00"
)

$ErrorActionPreference = "Continue"
$racine = Split-Path -Parent $PSScriptRoot
Set-Location $racine

# --- Lancement planifié : une seule fois par nuit, dans la plage horaire -----
# La tâche planifiée part à 23 h, mais aussi au déverrouillage et au réveil
# du PC : une Surface en veille moderne ne se réveille pas toujours à l'heure
# (nuit du 27 au 28/11 : départ à 00:52). Hors de la plage, ou si la nuit a
# déjà été lancée, on s'arrête tout de suite.
if ($Planifie) {
    $maintenant = Get-Date
    $hDebut = [datetime]::ParseExact($Heure, "HH:mm", $null).TimeOfDay
    $hFin = [datetime]::ParseExact($Fin, "HH:mm", $null).TimeOfDay
    $soir = $maintenant.Date
    if ($hFin -le $hDebut -and $maintenant.TimeOfDay -lt $hFin) { $soir = $soir.AddDays(-1) }
    $ouverture = $soir.Add($hDebut)
    $fermeture = $soir.Add($hFin)
    if ($fermeture -le $ouverture) { $fermeture = $fermeture.AddDays(1) }
    # Moins d'une heure avant la fin : pas la peine de commencer.
    if ($maintenant -lt $ouverture -or $maintenant -gt $fermeture.AddHours(-1)) { exit 0 }
    $baseNuit = Join-Path $racine "resultats\nuit"
    New-Item -ItemType Directory -Force -Path $baseNuit | Out-Null
    $marque = Join-Path $baseNuit "derniere-nuit.txt"
    $cle = $soir.ToString("yyyy-MM-dd")
    if ($Essai) { $cle += "-essai" }
    if ((Test-Path $marque) -and ((Get-Content $marque -TotalCount 1) -eq $cle)) { exit 0 }
    Set-Content -Path $marque -Value $cle -Encoding ASCII
}

# --- Empêcher la mise en veille tant que le script tourne -------------------
Add-Type -Namespace Fabbot -Name Veille -MemberDefinition @"
[System.Runtime.InteropServices.DllImport("kernel32.dll")]
public static extern uint SetThreadExecutionState(uint esFlags);
"@
# ES_CONTINUOUS | ES_SYSTEM_REQUIRED : le PC reste éveillé (l'écran peut s'éteindre).
[void][Fabbot.Veille]::SetThreadExecutionState([uint32]"0x80000001")

# --- Heure limite -----------------------------------------------------------
$debut = Get-Date
$limite = [datetime]::ParseExact($Fin, "HH:mm", $null)
$limite = $debut.Date.Add($limite.TimeOfDay)
if ($limite -le $debut) { $limite = $limite.AddDays(1) }

# --- Dossier du jour ----------------------------------------------------------
$base = Join-Path $racine "resultats\nuit"
New-Item -ItemType Directory -Force -Path $base | Out-Null
$nomJour = $debut.ToString("yyyy-MM-dd")
if ($Essai) { $nomJour += "-essai" }
$dossier = Join-Path $base $nomJour
$k = 2
while (Test-Path $dossier) { $dossier = Join-Path $base "$nomJour-$k"; $k++ }
New-Item -ItemType Directory -Force -Path $dossier | Out-Null
$resume = Join-Path $dossier "resume.md"

function Ecrire([string]$texte) {
    Add-Content -Path $resume -Value $texte -Encoding UTF8
    Write-Host $texte
}
function Duree([timespan]$t) {
    if ($t.TotalHours -ge 1) { return "{0} h {1:00} min" -f [int][math]::Floor($t.TotalHours), $t.Minutes }
    if ($t.TotalMinutes -ge 1) { return "{0} min {1:00} s" -f [int][math]::Floor($t.TotalMinutes), $t.Seconds }
    return "{0:0.0} s" -f $t.TotalSeconds
}

$titre = if ($Essai) { "Essai du script de nuit" } else { "Nuit du " + $debut.ToString("dd/MM/yyyy") }
Ecrire "# $titre"
Ecrire ""
Ecrire ("- Début : " + $debut.ToString("dd/MM/yyyy HH:mm") + " ; heure limite : " + $limite.ToString("dd/MM HH:mm"))
Ecrire ("- Machine : $env:COMPUTERNAME, " + [Environment]::ProcessorCount + " threads")
if ($Planifie) { Ecrire "- Lancement : tâche planifiée (plage $Heure à $Fin)" } else { Ecrire "- Lancement : à la main" }

# --- Mise à jour (seulement sur main, sans modification en cours) -----------
$git = Get-Command git -ErrorAction SilentlyContinue
if ($git) {
    $branche = (& git rev-parse --abbrev-ref HEAD 2>$null)
    $modifs = (& git status --porcelain --untracked-files=no 2>$null)
    if ($SansMiseAJour) {
        Ecrire "- Mise à jour : non demandée (-SansMiseAJour)"
    } elseif ($branche -ne "main") {
        Ecrire "- Mise à jour : **non faite**, le dossier est sur la branche « $branche » et pas sur main"
    } elseif ($modifs) {
        Ecrire "- Mise à jour : **non faite**, des fichiers suivis sont modifiés (git status)"
    } else {
        $sortie = (& git pull --ff-only -q 2>&1 | Out-String).Trim()
        if ($LASTEXITCODE -eq 0) { Ecrire "- Mise à jour : faite (git pull)" }
        else { Ecrire "- Mise à jour : **échec** ($sortie)" }
    }
    $commit = (& git log -1 --format="%h %ad %s" --date=format:"%d/%m %H:%M" 2>$null)
    Ecrire "- Version : $branche, $commit"
} else {
    Ecrire "- git introuvable : pas de mise à jour"
}

# --- Liste des travaux ---------------------------------------------------------
$aFaire = Join-Path $base "a-faire.txt"
if ($Essai) {
    $liste = Join-Path $racine "tools\nuit\essai.txt"
    Copy-Item $liste (Join-Path $dossier "travaux.txt")
} elseif (Test-Path $aFaire) {
    $liste = Join-Path $dossier "travaux.txt"
    Move-Item $aFaire $liste
    Ecrire "- Liste : resultats\nuit\a-faire.txt (liste du soir)"
} else {
    $liste = Join-Path $racine "tools\nuit\travaux.txt"
    Copy-Item $liste (Join-Path $dossier "travaux.txt")
    Ecrire "- Liste : tools\nuit\travaux.txt (liste par défaut)"
}
$travaux = @()
foreach ($l in (Get-Content -Path $liste -Encoding UTF8)) {
    $l = $l.Trim()
    if ($l -eq "" -or $l.StartsWith("#")) { continue }
    $p = $l.Split("|")
    if ($p.Count -lt 3) { Ecrire "- Ligne ignorée (format « nom | programme | arguments ») : $l"; continue }
    $travaux += [pscustomobject]@{ Nom = $p[0].Trim(); Prog = $p[1].Trim(); Args = ($p[2..($p.Count - 1)] -join "|").Trim() }
}
Ecrire ("- Travaux : " + $travaux.Count)

# --- Compilation ---------------------------------------------------------------
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargo) { Ecrire ""; Ecrire "**Arrêt : Rust (cargo) n'est pas installé.**"; exit 1 }
$logCompil = Join-Path $dossier "00-compilation.txt"
$t0 = Get-Date
& cmd.exe /c "cargo build --release -p fab-sim --bins > `"$logCompil`" 2>&1"
if ($LASTEXITCODE -ne 0) {
    Ecrire ""
    Ecrire "**Arrêt : la compilation a échoué** (voir 00-compilation.txt)."
    Ecrire '```'
    Get-Content $logCompil -Encoding UTF8 -Tail 30 | ForEach-Object { Ecrire $_ }
    Ecrire '```'
    exit 1
}
Ecrire ("- Compilation : " + (Duree ((Get-Date) - $t0)))

# --- Les travaux, un par un ------------------------------------------------------
$n = 0
$bilan = @()
foreach ($t in $travaux) {
    $n++
    $num = "{0:00}" -f $n
    $log = Join-Path $dossier "$num-$($t.Nom).txt"
    Ecrire ""
    Ecrire "## $num. $($t.Nom)"
    Ecrire ""
    Ecrire "``$($t.Prog) $($t.Args)``"
    Ecrire ""
    if ((Get-Date) -ge $limite) {
        Ecrire "Non lancé : heure limite atteinte."
        $bilan += "| $num | $($t.Nom) | non lancé (heure limite) | |"
        continue
    }
    $exe = Join-Path $racine "target\release\$($t.Prog).exe"
    if (-not (Test-Path $exe)) {
        Ecrire "Non lancé : programme « $($t.Prog) » introuvable."
        $bilan += "| $num | $($t.Nom) | programme introuvable | |"
        continue
    }
    # cmd /c : la sortie est copiée telle quelle (accents en UTF-8).
    $prio = if ($Discret) { "start `"`" /b /wait /belownormal " } else { "" }
    $ligne = "$prio`"$exe`" $($t.Args) > `"$log`" 2>&1"
    $tDebut = Get-Date
    $proc = Start-Process -FilePath "cmd.exe" -ArgumentList "/c", "`"$ligne`"" -WindowStyle Hidden -PassThru
    $arrete = $false
    while (-not $proc.WaitForExit(15000)) {
        if ((Get-Date) -ge $limite) {
            & taskkill.exe /T /F /PID $proc.Id | Out-Null
            $arrete = $true
            break
        }
    }
    $proc.WaitForExit()
    $d = Duree ((Get-Date) - $tDebut)
    if ($arrete) { $statut = "arrêté à l'heure limite" }
    elseif ($proc.ExitCode -eq 0) { $statut = "fini" }
    else { $statut = "**erreur (code $($proc.ExitCode))**" }
    Ecrire ("Début " + $tDebut.ToString("HH:mm") + ", durée $d : $statut.")
    $bilan += "| $num | $($t.Nom) | $statut | $d |"
    if (Test-Path $log) {
        $lignes = @(Get-Content $log -Encoding UTF8)
        Ecrire ""
        Ecrire '```'
        if ($lignes.Count -gt 60) {
            Ecrire "[... $($lignes.Count - 40) premières lignes dans $num-$($t.Nom).txt ...]"
            $lignes = $lignes[($lignes.Count - 40)..($lignes.Count - 1)]
        }
        $lignes | ForEach-Object { Ecrire $_ }
        Ecrire '```'
    }
}

# --- Bilan ---------------------------------------------------------------------------
$finNuit = Get-Date
Ecrire ""
Ecrire "## Bilan"
Ecrire ""
Ecrire "| # | Travail | Statut | Durée |"
Ecrire "|---|---|---|---|"
$bilan | ForEach-Object { Ecrire $_ }
Ecrire ""
Ecrire ("Fin : " + $finNuit.ToString("dd/MM/yyyy HH:mm") + ", durée totale " + (Duree ($finNuit - $debut)) + ".")
[void][Fabbot.Veille]::SetThreadExecutionState([uint32]"0x80000000")
