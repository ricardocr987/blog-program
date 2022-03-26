use anchor_lang::prelude::*;

declare_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

#[program]
/// Moving on to the #[program] macro below, this is where we define our program.
/// Each method inside here defines an RPC request handler (aka instruction handler) which can be invoked by clients
pub mod blog_program {
    use super::*;

    pub fn initialize_blog(
        // El primer parámetro para cada RPC handler/instrucción es el Context struct
        ctx: Context<InitializeBlog>, 
        blog_account_bump: u8,
        vector_capacity: u16,
        category: String,
    ) -> Result<()> {

        let blog = &mut ctx.accounts.blog_account; // declaramos una refecencia mutable del account

        blog.bump = blog_account_bump; // Almacenamos el bump!
        blog.authority = *ctx.accounts.authority.to_account_info().key; // * actúa para deshacer la referencia, es lo contrario a &
                // simplemente con este * nos aseguramos de que estamos guardando el valor en vez del puntero (esto es Rust)
        blog.post_count = 0;
        blog.category = category;
        blog.vector_capacity = vector_capacity;
        blog.subscribers_pubkeys = Vec::new(); // Así es como se inicializa un vector

        Ok(())
    }

    pub fn update_blog(
        ctx: Context<InitializeBlog>, 
        category: String,
    ) -> Result<()> {

        let blog = &mut ctx.accounts.blog_account;

        blog.category = category;

        Ok(())
    }

    pub fn add_subscriber(
        ctx: Context<InitializeBlog>, 
        sub_pubkey: Pubkey,
    ) -> Result<()> {

        let blog = &mut ctx.accounts.blog_account;

        blog.subscribers_pubkeys.push(sub_pubkey); // Añadimos la pubkey del subscriptor a nuestro vector

        Ok(())
    }

    pub fn create_post(
        ctx: Context<CreatePost>, 
        post_account_bump: u8, 
        title: String, 
        body: String
    ) -> Result<()> {

        let blog = &mut ctx.accounts.blog_account;
        let post = &mut ctx.accounts.post_account;

        post.authority = *ctx.accounts.authority.to_account_info().key;
        post.title = title;
        post.body = body;
        post.bump = post_account_bump;
        post.entry = blog.post_count;
       
        blog.post_count += 1;

        Ok(())
    }

    pub fn update_post(
        ctx: Context<UpdatePost>, 
        title: String, 
        body: String
    ) -> Result<()> {

        let post = &mut ctx.accounts.post_account;

        post.title = title;
        post.body = body;

        Ok(())
    }

    pub fn delete_post(
        ctx: Context<UpdatePost>, 
        title: String,
        body: String
    ) -> Result<()> {

        let post = &mut ctx.accounts.post_account;

        post.title = title;
        post.body = body;

        Ok(())
    }
}

#[derive(Accounts)]
// Puedes acceder a los argumentos de las instruccion con #[instruction(..)]
// Tiene que ser el mismo orden que en la instrucción, se suele usar para almacenar el bump 
// y para establecer el espacio que ocupa variables dinámicas (String y Vectores)
#[instruction(blog_account_bump: u8, vector_capacity: u16, category: String)]
pub struct InitializeBlog<'info> {
    #[account(
        init, // Permite inicializar la Account
        payer = authority, // Especifíca quien va a pagar el rent
        space = BlogAccount::space(vector_capacity, &category),
        seeds = [ // Nuestra Account tendrá un PDA derivado de las seeds "blog" + authority pubkey
            // Especifíca y comprueba las seeds para el PDA de la Account con init, 
            // combinado con init comprueba si el PDA ya existe
            b"blog".as_ref(), // as_ref() es un método que convierte código genérico en una referencia
            authority.key().as_ref(), // Esto es Rust: as_ref() es un método que nos permite pedir prestado una referencia
            // Léete esta capítulo del libro de Rust: https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html
        ],
        bump, // Valor que completa a las seeds para obtener un PDA válido
            // en tutoriales anteriores verás como antes se igualaba con la variable que hay en #[instruction(..)]
            // ahora ya se hace automáticamente
    )]
    blog_account: Account<'info, BlogAccount>,
    #[account(mut)]
    authority: Signer<'info>,
    system_program: Program<'info, System> // Es requerido cuando inicializamos una Account
        // Esto exige que nuestra Account deba ser propiedad de nuestro Program y que debe ser deserializado
        // como el struct BlogAccount de abajo en #[account] 
}

#[derive(Accounts)]
pub struct UpdateBlog<'info> {
    #[account(mut, has_one = authority)]
    blog_account: Account<'info, BlogAccount>,
    #[account(mut)]
    authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct AddSubscriber<'info> {
    #[account(mut, has_one = authority)]
    blog_account: Account<'info, BlogAccount>,
    #[account(mut)]
    authority: Signer<'info>,
}

#[derive(Accounts)]
#[instruction(post_account_bump: u8, title: String, body: String)]
pub struct CreatePost<'info> {
    #[account(
        init, 
        payer = authority, 
        seeds = [ // cada post tendrá un PDA derivado de ["post" + Account Blog pubkey (que tmb es un PDA) + post_count]
            b"post".as_ref(), 
            blog_account.key().as_ref(),
            &[blog_account.post_count as u8].as_ref(),
        ],
        bump, 
        space = PostAccount::space(&title, &body), // Dice cuando espacio va a ocupar
                    // la Account, necesario para calcular la cantidad de rent-exemption
                    // llamamos a la función space
    )]
    post_account: Account<'info, PostAccount>,
    // Todas las validaciones son manejadas por #[account(..)] macros, permitiéndonos centrarnos en la lógica o en el diseño de las instrucciones
    #[account(mut, has_one = authority)] // has_one = nos permite implementar una restricción en la account para que solo el usuario pueda modificarla
    blog_account: Account<'info, BlogAccount>,
    #[account(mut)]
    authority: Signer<'info>,
    system_program: Program<'info, System>
}

#[derive(Accounts)]
pub struct UpdatePost<'info> {
    #[account(mut, has_one = authority)]
    post_account: Account<'info, PostAccount>,
    #[account(mut)]
    authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct DeletePost<'info> {
    #[account(mut, has_one = authority)]
    post_account: Account<'info, PostAccount>,
    #[account(mut)]
    authority: Signer<'info>,
}

/// Aquí definimos nuetro Blog Account
/// Definimos un struct que tiene tres propiedades públicas: bump, post_count y authority
/// Esta Account será pasada como argumento a las instrucciones si la incluimos en el Context
#[account]
pub struct BlogAccount {
    pub bump: u8, // int de 1byte/8bits, bump: Nos dice el valor que completa a las seeds para obtener un PDA válido
    pub post_count: u8,  // int para almacenar el número de posts que tiene cada blog
    pub authority: Pubkey, // almacenamos la Pubkey del usuario propietario del blog
    pub category: String, // almacenamos el tema/categoria sobre el blog del usuario
    pub vector_capacity: u16, // Guarmos la capacidad máxima del vector
    pub subscribers_pubkeys: Vec<Pubkey>, // Guardamos las pubKeys de los usuarios que estén suscritos al blog
}

impl BlogAccount { // Cada struct puede tener múltiple impl blocks, básicamente puedes declarar funciones
    pub fn space (capacity: u16, category: &str) -> usize { // Esta función devuelve un integer unsigned (sólo permite positivos)
        // los strings son referencias, léete esta capítulo del libro de Rust: https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html
        8 + // Discrimanator: son 8 bytes que Anchor pone al frente de una cuenta, como una cabecera. Permite a Anchor saber 
        // qué tipo de cuenta debe deserializar los datos.
        1 + // u8
        32 + // Pubkey
        4 + category.len() + // el 4 es otro discriminator, + la longitud de la cadena, esto genera un problema, una vez que la Account es creada
        // el usuario no podría poner una categoría más larga de lo que puso en un primer momento, esto se podría arreglar especificando un 
        // espacio extra, pero como esto es un ejemplo tampoco es necesario optimizarlo tanto, pero haré un test que compruebe esta situación
        4 + (capacity as usize) * std::mem::size_of::<Pubkey>()
        // Para sacar el espacio que ocupa un vector tenemos que multiplacar la capacidad que asignamos nosotros por el espacio de la variable que 
        // queremos vectorizar. La capacidad representa cuantas posiciones podemos almacenar dentro del vector.
    }
}


#[account]
pub struct PostAccount{
    pub authority: Pubkey,
    pub bump: u8,
    pub entry: u8,
    pub title: String,
    pub body: String,
}

impl PostAccount{ 
    pub fn space(title: &str, body: &str) -> usize{ 
        8 + // discriminator
        32 + // Pubkey
        1 + // u8
        1 + // u8
        4 + title.len() + // String
        4 + body.len() // String
    }
}
