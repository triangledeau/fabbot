package fr.arsenal.fabbot

import android.annotation.SuppressLint
import android.content.ActivityNotFoundException
import android.content.Intent
import android.graphics.Color
import android.net.Uri
import android.os.Bundle
import android.util.Log
import android.view.Gravity
import android.view.View
import android.webkit.JsResult
import android.webkit.WebChromeClient
import android.webkit.WebResourceRequest
import android.webkit.WebView
import android.webkit.WebViewClient
import android.widget.FrameLayout
import android.widget.TextView
import android.widget.Toast
import androidx.activity.OnBackPressedCallback
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import androidx.core.content.ContextCompat
import androidx.core.view.ViewCompat
import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import java.io.File
import java.io.FileNotFoundException

/**
 * La seule activité de l'appli : une WebView plein écran qui affiche la
 * page d'Arsenal, servie par le moteur Rust à l'adresse
 * `http://127.0.0.1:<port>/` (voir docs/android.md, section 3).
 */
class ActiviteArsenal : AppCompatActivity() {

    private lateinit var page: WebView
    private lateinit var attente: TextView
    private var port: Int = 0

    @SuppressLint("SetJavaScriptEnabled")
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Depuis Android 15, l'appli dessine sous les barres du système :
        // on laisse leur place en marge, sur le fond de la page.
        WindowCompat.setDecorFitsSystemWindows(window, false)
        val fond = ContextCompat.getColor(this, R.color.fond)
        val cadre = FrameLayout(this)
        cadre.setBackgroundColor(fond)

        page = WebView(this)
        page.setBackgroundColor(fond)
        page.settings.javaScriptEnabled = true
        page.settings.domStorageEnabled = true // réglages mémorisés (localStorage)
        page.settings.allowFileAccess = false
        page.settings.allowContentAccess = false
        page.webViewClient = ClientPage()
        page.webChromeClient = ClientDialogues()
        cadre.addView(page, FrameLayout.LayoutParams(-1, -1))

        attente = TextView(this)
        attente.text = "Chargement d’Arsenal…"
        attente.setTextColor(Color.rgb(0xC9, 0xCF, 0xDA))
        attente.textSize = 18f
        attente.gravity = Gravity.CENTER
        cadre.addView(attente, FrameLayout.LayoutParams(-1, -1))

        ViewCompat.setOnApplyWindowInsetsListener(cadre) { v, encarts ->
            val barres = encarts.getInsets(
                WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout()
            )
            val clavier = encarts.getInsets(WindowInsetsCompat.Type.ime())
            v.setPadding(barres.left, barres.top, barres.right, maxOf(barres.bottom, clavier.bottom))
            WindowInsetsCompat.CONSUMED
        }
        setContentView(cadre)
        WindowCompat.getInsetsController(window, cadre).apply {
            isAppearanceLightStatusBars = false
            isAppearanceLightNavigationBars = false
        }

        // Bouton Retour : revient en arrière dans la page si elle le peut,
        // sinon quitte l'appli.
        onBackPressedDispatcher.addCallback(this, object : OnBackPressedCallback(true) {
            override fun handleOnBackPressed() {
                if (page.canGoBack()) page.goBack() else finish()
            }
        })

        // Copie des textes des cartes et démarrage du moteur hors du fil
        // de l'écran (la copie prend une seconde au premier lancement).
        Thread({
            val resultat = try {
                installerTextes()
                Moteur.demarrer(filesDir.absolutePath)
            } catch (e: Throwable) {
                Log.e(ETIQUETTE, "Démarrage impossible", e)
                -1
            }
            runOnUiThread { afficher(resultat) }
        }, "demarrage").start()
    }

    private fun afficher(resultat: Int) {
        if (isFinishing || isDestroyed) return
        if (resultat <= 0) {
            attente.text = "Arsenal n’a pas pu démarrer.\n\n" +
                "Le détail est dans le fichier sauvegardes/plantages.txt " +
                "de l’appli, ou dans le journal du téléphone " +
                "(adb logcat -s fab-android)."
            return
        }
        port = resultat
        page.loadUrl("http://127.0.0.1:$port/")
    }

    /**
     * Copie le fichier des textes officiels des cartes, rangé dans l'APK,
     * dans le dossier de l'appli, où le moteur le lit
     * (`fab_android::chemin_textes`). Seulement au premier lancement et
     * après chaque mise à jour de l'appli.
     */
    private fun installerTextes() {
        val dossier = File(filesDir, "donnees")
        val cible = File(dossier, "public_card_data.csv")
        val repere = File(dossier, "version-textes.txt")
        val version = packageManager.getPackageInfo(packageName, 0).lastUpdateTime.toString()
        if (cible.isFile && repere.isFile && repere.readText() == version) return
        dossier.mkdirs()
        val provisoire = File(dossier, "public_card_data.csv.part")
        try {
            assets.open("donnees/public_card_data.csv").use { entree ->
                provisoire.outputStream().use { sortie -> entree.copyTo(sortie, 1 shl 16) }
            }
        } catch (e: FileNotFoundException) {
            Log.w(ETIQUETTE, "Textes des cartes absents de l'APK : la page marche sans eux.")
            provisoire.delete()
            return
        }
        if (!provisoire.renameTo(cible)) {
            cible.delete()
            provisoire.renameTo(cible)
        }
        repere.writeText(version)
    }

    override fun onResume() {
        super.onResume()
        page.onResume()
    }

    override fun onPause() {
        page.onPause()
        super.onPause()
    }

    override fun onDestroy() {
        page.destroy()
        super.onDestroy()
    }

    /** Les pages d'Arsenal restent dans l'appli ; les autres liens s'ouvrent ailleurs. */
    private inner class ClientPage : WebViewClient() {
        override fun shouldOverrideUrlLoading(view: WebView, requete: WebResourceRequest): Boolean {
            val adresse = requete.url
            val schema = adresse.scheme ?: return false
            if (schema != "http" && schema != "https") return false
            if (adresse.host == "127.0.0.1" || adresse.host == "localhost") return false
            try {
                startActivity(Intent(Intent.ACTION_VIEW, Uri.parse(adresse.toString())))
            } catch (e: ActivityNotFoundException) {
                Toast.makeText(this@ActiviteArsenal, R.string.lien_impossible, Toast.LENGTH_SHORT).show()
            }
            return true
        }

        override fun onPageFinished(view: WebView, adresse: String) {
            if (adresse.startsWith("http://127.0.0.1")) attente.visibility = View.GONE
        }
    }

    /** `alert(…)` et `confirm(…)` de la page, en boîtes de dialogue Android, en français. */
    private inner class ClientDialogues : WebChromeClient() {
        override fun onJsAlert(view: WebView, adresse: String, message: String, reponse: JsResult): Boolean {
            AlertDialog.Builder(this@ActiviteArsenal)
                .setMessage(message)
                .setPositiveButton(R.string.ok) { _, _ -> reponse.confirm() }
                .setOnCancelListener { reponse.confirm() }
                .show()
            return true
        }

        override fun onJsConfirm(view: WebView, adresse: String, message: String, reponse: JsResult): Boolean {
            AlertDialog.Builder(this@ActiviteArsenal)
                .setMessage(message)
                .setPositiveButton(R.string.ok) { _, _ -> reponse.confirm() }
                .setNegativeButton(R.string.annuler) { _, _ -> reponse.cancel() }
                .setOnCancelListener { reponse.cancel() }
                .show()
            return true
        }
    }

    companion object {
        const val ETIQUETTE = "fab-android"
    }
}
